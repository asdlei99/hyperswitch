# MONEI Connector Implementation: Final Summary

## Overview

This document provides a comprehensive analysis of the current MONEI connector implementation in Hyperswitch, comparing it with the actual MONEI API documentation. It consolidates the findings from our detailed review and outlines the necessary changes to ensure the connector works correctly with the MONEI payment gateway.

## Current Implementation Status

The current implementation includes most of the core functionality required for integrating with MONEI:

| Feature | Status | Notes |
|---------|--------|-------|
| Authentication | ✅ Implemented | Using Bearer token format correctly |
| Payment Authorization | ✅ Implemented | Basic functionality works but field names differ |
| Payment Sync | ✅ Implemented | Basic functionality works |
| Payment Capture | ⚠️ Partially Correct | Wrong URL structure and request format |
| Payment Void | ❌ Incomplete | Method declared but not fully implemented |
| Refund Execution | ⚠️ Partially Correct | Wrong URL structure and request format |
| Refund Sync | ✅ Implemented | Basic functionality works |
| Error Handling | ✅ Implemented | Comprehensive implementation |
| Tests | ✅ Implemented | Comprehensive test suite |

## Key Issues Identified

### 1. Endpoint URL Structure

The current implementation uses incorrect URL structures for several operations:

```rust
// Current
pub const CAPTURES_URL: &str = "/captures";
pub const REFUNDS_URL: &str = "/refunds";

// Should be
// Capture: /payments/:id/capture
// Refund: /payments/:id/refund
// Cancel: /payments/:id/cancel
```

This affects how capture and refund requests are constructed. Instead of standalone endpoints, these operations should be performed as sub-resources of a specific payment.

### 2. Request and Response Structures

Several discrepancies exist between the implemented data structures and MONEI's API requirements:

#### Payment Request:
- Field naming: The implementation uses `reference` instead of `orderId`, `complete` instead of `transactionType`, etc.
- Missing fields: Several fields like `callbackUrl`, `completeUrl`, etc. are not included in the request structure.

#### Capture Request:
```rust
// Current
{
  "payment": "payment_id",
  "amount": "100"
}

// Should be (with payment_id in URL)
{
  "amount": 100
}
```

#### Refund Request:
```rust
// Current
{
  "payment": "payment_id",
  "amount": "100",
  "reason": "requested_by_customer",
  "reference": "ref123"
}

// Should be (with payment_id in URL)
{
  "amount": 100,
  "refundReason": "requested_by_customer"
}
```

### 3. Payment Status Handling

The status mapping is mostly correct but missing the `EXPIRED` status:

```rust
// Current implementation
match item {
    MoneiPaymentStatus::AUTHORIZED => Self::Authorized,
    MoneiPaymentStatus::SUCCEEDED => Self::Charged,
    MoneiPaymentStatus::FAILED => Self::Failure,
    MoneiPaymentStatus::PENDING => Self::Pending,
    MoneiPaymentStatus::CANCELED => Self::Voided,
    MoneiPaymentStatus::REFUNDED => Self::Charged,
    MoneiPaymentStatus::PartiallyRefunded => Self::Charged,
}

// Should add
// MoneiPaymentStatus::EXPIRED => Self::AuthorizationFailed,
```

### 4. Missing Functionality

The `PaymentVoid` trait is implemented but the actual functionality is not fully implemented in the `ConnectorIntegration<Void, PaymentsCancelData, PaymentsResponseData>` implementation.

### 5. Authentication Method

The documentation in MONEI_specs.md mentions `ConnectorAuthType::BodyKey`, but the actual implementation uses `ConnectorAuthType::HeaderKey`. The implementation appears to be correct since the API key is sent in the Authorization header, but the documentation should be updated.

## Implementation Recommendations

### Priority 1: Fix Critical URL Structures

1. Update the URL constants to match the actual API endpoints:
   ```rust
   pub const PAYMENTS_URL: &str = "/payments";
   pub const CAPTURE_SUFFIX: &str = "/capture";
   pub const REFUND_SUFFIX: &str = "/refund";
   pub const CANCEL_SUFFIX: &str = "/cancel";
   ```

2. Update the URL construction in `get_url` methods:
   ```rust
   // For capture
   fn get_url(&self, req: &PaymentsCaptureRouterData, connectors: &Connectors) -> CustomResult<String, errors::ConnectorError> {
       let connector_payment_id = req.request.connector_transaction_id.clone();
       Ok(format!(
           "{}{}{}{}",
           self.base_url(connectors),
           PAYMENTS_URL,
           connector_payment_id,
           CAPTURE_SUFFIX
       ))
   }
   
   // Similar updates for refund and cancel
   ```

### Priority 2: Fix Request Structures

1. Update the `MoneiPaymentsRequest` structure:
   ```rust
   pub struct MoneiPaymentsRequest {
       amount: StringMinorUnit,
       currency: String,
       #[serde(rename = "orderId")]
       order_id: Option<String>,
       description: Option<String>,
       #[serde(rename = "paymentMethod")]
       payment_method: MoneiPaymentMethod,
       customer: Option<MoneiCustomer>,
       #[serde(rename = "transactionType")]
       transaction_type: String, // "SALE" or "AUTH"
       #[serde(rename = "completeUrl", skip_serializing_if = "Option::is_none")]
       complete_url: Option<String>,
       // Add other URL fields
   }
   ```

2. Update the `TryFrom` implementation to set `transaction_type` based on capture preference:
   ```rust
   transaction_type: if is_auto_capture { "SALE" } else { "AUTH" }.to_string(),
   ```

3. Update the `MoneiRefundRequest` structure:
   ```rust
   pub struct MoneiRefundRequest {
       pub amount: StringMinorUnit,
       #[serde(rename = "refundReason", skip_serializing_if = "Option::is_none")]
       pub refund_reason: Option<String>,
   }
   ```

### Priority 3: Implement Payment Void

Fully implement the `PaymentVoid` trait for canceling authorized payments:

```rust
impl ConnectorIntegration<Void, PaymentsCancelData, PaymentsResponseData> for Monei {
    fn get_headers(
        &self,
        req: &PaymentsCancelRouterData,
        connectors: &Connectors,
    ) -> CustomResult<Vec<(String, masking::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &PaymentsCancelRouterData,
        connectors: &Connectors,
    ) -> CustomResult<String, errors::ConnectorError> {
        let connector_payment_id = req.request.connector_transaction_id.clone();
        Ok(format!(
            "{}{}{}{}",
            self.base_url(connectors),
            PAYMENTS_URL,
            connector_payment_id,
            CANCEL_SUFFIX
        ))
    }

    // Implement other required methods
}
```

### Priority 4: Update Status Handling

Add handling for the `EXPIRED` status:

```rust
impl From<MoneiPaymentStatus> for common_enums::AttemptStatus {
    fn from(item: MoneiPaymentStatus) -> Self {
        match item {
            // Existing mappings
            MoneiPaymentStatus::EXPIRED => Self::AuthorizationFailed,
        }
    }
}
```

### Additional Considerations

1. **Authentication Method**: Review if `HeaderKey` is the correct authentication type. The implementation seems correct, but the documentation mentions `BodyKey`.

2. **Confirm Payment Flow**: Consider if implementing the Confirm Payment flow is necessary for the use cases Hyperswitch supports.

3. **Error Messages**: Ensure error message handling is consistent with the actual MONEI API error responses.

## Testing Recommendations

The existing test suite is comprehensive and covers most scenarios. After implementing the changes, run the full test suite to ensure all functionality continues to work correctly. Pay special attention to:

1. Capture and refund tests with the new URL structures
2. Cancel/void payment tests with the proper implementation
3. Error handling tests to ensure consistent behavior

## Conclusion

The current MONEI connector implementation provides a solid foundation but requires several adjustments to correctly match the MONEI API specifications. By implementing the recommended changes, the connector will be fully aligned with the MONEI API and provide a robust integration for processing payments through this gateway.
