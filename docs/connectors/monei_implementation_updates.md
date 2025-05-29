# MONEI Connector Implementation Updates

Based on a detailed comparison between the actual MONEI API documentation and the current implementation, this document outlines key discrepancies and recommended updates to bring the implementation in line with the actual API specifications.

## 1. URL Structure Discrepancies

### Current Implementation
```rust
pub const BASE_URL: &str = "https://api.monei.com/v1";
pub const PAYMENTS_URL: &str = "/payments";
pub const CAPTURES_URL: &str = "/captures";
pub const REFUNDS_URL: &str = "/refunds";
```

### Actual API Endpoints
```
- Create Payment: POST https://api.monei.com/v1/payments
- Get Payment: GET https://api.monei.com/v1/payments/:id
- Capture Payment: POST https://api.monei.com/v1/payments/:id/capture
- Cancel Payment: POST https://api.monei.com/v1/payments/:id/cancel
- Refund Payment: POST https://api.monei.com/v1/payments/:id/refund
```

### Required Changes
- Update `CAPTURES_URL` to `/payments/:id/capture`
- Update `REFUNDS_URL` to `/payments/:id/refund`
- Add `CANCEL_URL` as `/payments/:id/cancel`

## 2. Payment Request Structure

### Current Implementation
```rust
pub struct MoneiPaymentsRequest {
    amount: StringMinorUnit,
    currency: String,
    reference: Option<String>,
    description: Option<String>,
    payment_method: MoneiPaymentMethod,
    customer: Option<MoneiCustomer>,
    complete: bool,
    return_url: Option<String>,
}
```

### Actual API Structure
```json
{
  "amount": 110,
  "currency": "EUR",
  "orderId": "14379133960355",
  "callbackUrl": "https://example.com/checkout/callback",
  "completeUrl": "https://example.com/checkout/complete",
  "failUrl": "https://example.com/checkout/fail",
  "cancelUrl": "https://example.com/checkout/cancel",
  "paymentToken": "7cc38b08ff471ccd313ad62b23b9f362b107560b",
  "sessionId": "39603551437913",
  "generatePaymentToken": false,
  "paymentMethod": {
    "card": {
      "number": "string",
      "cvc": "string",
      "expMonth": "string",
      "expYear": "string",
      "cardholderName": "John Doe",
      "cardholderEmail": "john.doe@monei.com"
    }
  },
  "allowedPaymentMethods": ["card", "bizum", "paypal"],
  "transactionType": "SALE",
  "description": "Test Shop - #84370745531439",
  "customer": {
    "email": "john.doe@example.com",
    "name": "John Doe",
    "phone": null
  },
  "billingDetails": {
    "name": "John Doe",
    "email": "john.doe@example.com",
    "phone": null,
    "address": {
      "country": "ES",
      "city": "Málaga",
      "line1": "Fake Street 123",
      "line2": null,
      "zip": "1234",
      "state": "Málaga"
    }
  }
}
```

### Required Changes
- Rename `reference` to `orderId`
- Replace `complete: bool` with `transactionType: String` (values: "SALE" or "AUTH")
- Update field names to match API documentation (e.g., `expiry_month` to `expMonth`)
- Add support for various URL fields (callbackUrl, completeUrl, failUrl, cancelUrl)
- Add support for `generatePaymentToken` field

## 3. Capture Request Implementation

### Current Implementation
```rust
// URL: POST to "/captures"
let capture_request = serde_json::json!({
    "payment": payment_id,
    "amount": amount
});
```

### Actual API Structure
```
// URL: POST to "/payments/:id/capture"
{
  "amount": 110
}
```

### Required Changes
- Update URL to use `/payments/:id/capture` format
- Simplify request body to only include the amount

## 4. Refund Request Implementation

### Current Implementation
```rust
// URL: POST to "/refunds"
pub struct MoneiRefundRequest {
    pub payment: String,
    pub amount: StringMinorUnit,
    pub reason: Option<String>,
    pub reference: Option<String>,
}
```

### Actual API Structure
```
// URL: POST to "/payments/:id/refund"
{
  "amount": 110,
  "refundReason": null
}
```

### Required Changes
- Update URL to use `/payments/:id/refund` format
- Update field name from `reason` to `refundReason`
- Remove `payment` field from body as it's part of the URL
- Remove `reference` field as it's not part of the API

## 5. Payment Status Handling

### Current Implementation
```rust
impl From<MoneiPaymentStatus> for common_enums::AttemptStatus {
    fn from(item: MoneiPaymentStatus) -> Self {
        match item {
            MoneiPaymentStatus::AUTHORIZED => Self::Authorized,
            MoneiPaymentStatus::SUCCEEDED => Self::Charged,
            MoneiPaymentStatus::FAILED => Self::Failure,
            MoneiPaymentStatus::PENDING => Self::Pending,
            MoneiPaymentStatus::CANCELED => Self::Voided,
            MoneiPaymentStatus::REFUNDED => Self::Charged,
            MoneiPaymentStatus::PartiallyRefunded => Self::Charged,
        }
    }
}
```

### Required Update
- Add handling for `EXPIRED` status, mapping to `Self::AuthorizationFailed`

## 6. Missing Functionality: Payment Void (Cancel)

### Required Implementation
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
            "{}{}/{}/cancel",
            self.base_url(connectors),
            PAYMENTS_URL,
            connector_payment_id
        ))
    }

    fn get_request_body(
        &self,
        req: &PaymentsCancelRouterData,
        _connectors: &Connectors,
    ) -> CustomResult<RequestContent, errors::ConnectorError> {
        let cancel_reason = req.request.cancellation_reason.clone();
        
        let cancel_request = if let Some(reason) = cancel_reason {
            serde_json::json!({
                "cancellationReason": reason
            })
        } else {
            serde_json::json!({})
        };
        
        Ok(RequestContent::Json(Box::new(cancel_request)))
    }

    // Add the remaining required methods...
}
```

## 7. Authentication Method

### Current Implementation
```rust
impl TryFrom<&ConnectorAuthType> for MoneiAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            ConnectorAuthType::HeaderKey { api_key } => Ok(Self {
                api_key: api_key.to_owned(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}
```

### Required Update
- The documentation in MONEI_specs.md mentions `ConnectorAuthType::BodyKey`, but the actual implementation uses `ConnectorAuthType::HeaderKey`. Verify which is correct based on how the API key is sent to MONEI.

## 8. Additional Feature: Confirm Payment Flow

### Potential Implementation
Consider implementing the Confirm Payment flow if required by the use cases:

```rust
// Add a new constant
pub const CONFIRM_URL: &str = "/confirm";

// Implement the confirm payment request structure
pub struct MoneiConfirmPaymentRequest {
    paymentToken: String,
    generatePaymentToken: Option<bool>,
    // Add other fields...
}

// Implement ConnectorIntegration for a new ConfirmPayment flow type
impl ConnectorIntegration<ConfirmPayment, PaymentsConfirmData, PaymentsResponseData> for Monei {
    // Implement required methods...
}
```

## Conclusion

The current implementation has a solid foundation but requires several adjustments to correctly match the MONEI API specifications. The most critical changes are:

1. Fixing the URL structures for capture and refund operations
2. Updating request/response field names to match the API
3. Implementing the missing payment void (cancel) functionality
4. Enhancing status and error handling for completeness

These changes will ensure the connector works correctly with the MONEI payment gateway across all supported operations.
