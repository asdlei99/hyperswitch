# MONEI Connector Implementation: Step-by-Step Plan

This document provides a detailed step-by-step implementation plan for updating the MONEI connector to align with the actual API specifications. Each phase is designed to be atomic and independently compilable, with verification steps to ensure the code builds successfully.

## Phase 1: URL Structure Updates

### Step 1.1: Update URL Constants

**File**: `crates/hyperswitch_connectors/src/connectors/monei.rs`

```rust
// Current
pub const BASE_URL: &str = "https://api.monei.com/v1";
pub const PAYMENTS_URL: &str = "/payments";
pub const CAPTURES_URL: &str = "/captures";
pub const REFUNDS_URL: &str = "/refunds";

// Change to
pub const BASE_URL: &str = "https://api.monei.com/v1";
pub const PAYMENTS_URL: &str = "/payments";
pub const CAPTURE_SUFFIX: &str = "/capture";
pub const REFUND_SUFFIX: &str = "/refund";
pub const CANCEL_SUFFIX: &str = "/cancel";
```

### Step 1.2: Update Capture URL Method

**File**: `crates/hyperswitch_connectors/src/connectors/monei.rs`

```rust
// Replace the current get_url method in ConnectorIntegration<Capture, ...> implementation
fn get_url(
    &self,
    req: &PaymentsCaptureRouterData,
    connectors: &Connectors,
) -> CustomResult<String, errors::ConnectorError> {
    let connector_payment_id = req.request.connector_transaction_id.clone();
    Ok(format!(
        "{}{}/{}{}",
        self.base_url(connectors),
        PAYMENTS_URL,
        connector_payment_id,
        CAPTURE_SUFFIX
    ))
}
```

### Step 1.3: Update Refund URL Method

**File**: `crates/hyperswitch_connectors/src/connectors/monei.rs`

```rust
// Replace the current get_url method in ConnectorIntegration<Execute, ...> implementation
fn get_url(
    &self,
    req: &RefundsRouterData<Execute>,
    connectors: &Connectors,
) -> CustomResult<String, errors::ConnectorError> {
    let connector_payment_id = req.request.connector_transaction_id.clone();
    Ok(format!(
        "{}{}/{}{}",
        self.base_url(connectors),
        PAYMENTS_URL,
        connector_payment_id,
        REFUND_SUFFIX
    ))
}
```

### Step 1.4: Update Refund Sync URL Method

**File**: `crates/hyperswitch_connectors/src/connectors/monei.rs`

```rust
// Replace the current get_url method in ConnectorIntegration<RSync, ...> implementation
fn get_url(
    &self,
    req: &RefundSyncRouterData,
    connectors: &Connectors,
) -> CustomResult<String, errors::ConnectorError> {
    let refund_id = req.request.get_connector_refund_id()?;
    let connector_payment_id = req.request.connector_transaction_id.clone();
    Ok(format!(
        "{}{}/{}/refunds/{}",
        self.base_url(connectors),
        PAYMENTS_URL,
        connector_payment_id,
        refund_id
    ))
}
```

### Step 1.5: Verify Phase 1 Builds Correctly

```bash
cargo build
```

## Phase 2: Request Structure Updates

### Step 2.1: Update Payment Request Structure

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update MoneiPaymentsRequest struct
#[derive(Default, Debug, Serialize, PartialEq)]
pub struct MoneiPaymentsRequest {
    /// The payment amount in minor units (e.g. cents for USD)
    amount: StringMinorUnit,
    /// Currency code in ISO 4217 format
    currency: String,
    /// A unique identifier for the payment
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    order_id: Option<String>,
    /// Description of the payment shown to the customer
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Payment method details (card for card payments)
    #[serde(rename = "paymentMethod")]
    payment_method: MoneiPaymentMethod,
    /// Customer information
    #[serde(skip_serializing_if = "Option::is_none")]
    customer: Option<MoneiCustomer>,
    /// Controls when funds will be captured (SALE or AUTH)
    #[serde(rename = "transactionType")]
    transaction_type: String,
    /// Return URL where the customer will be redirected after the payment
    #[serde(rename = "completeUrl", skip_serializing_if = "Option::is_none")]
    complete_url: Option<String>,
}
```

### Step 2.2: Update Card Fields

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update MoneiCard struct
#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MoneiCard {
    /// Card number
    number: cards::CardNumber,
    /// Expiry month (MM format)
    #[serde(rename = "expMonth")]
    expiry_month: Secret<String>,
    /// Expiry year (YY format)
    #[serde(rename = "expYear")]
    expiry_year: Secret<String>,
    /// Card security code
    cvc: Secret<String>,
    /// Cardholder name
    #[serde(rename = "cardholderName", skip_serializing_if = "Option::is_none")]
    cardholder_name: Option<String>,
    /// Cardholder email
    #[serde(rename = "cardholderEmail", skip_serializing_if = "Option::is_none")]
    cardholder_email: Option<String>,
}
```

### Step 2.3: Update TryFrom Implementation for Payment Request

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update the TryFrom implementation
impl TryFrom<&MoneiRouterData<&PaymentsAuthorizeRouterData>> for MoneiPaymentsRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &MoneiRouterData<&PaymentsAuthorizeRouterData>) -> Result<Self, Self::Error> {
        let router_data = item.router_data;
        let request = &router_data.request;
        let is_auto_capture = request.is_auto_capture()?;
        
        // Extract email for customer details
        let customer = request.email.as_ref().map(|_| {
            // Get the email address as a string, or use an empty string if not available
            let email_str = request.email.clone()
                .map(|e| format!("{:?}", e).replace("\"", ""))
                .unwrap_or_default();
            
            MoneiCustomer {
                email: Some(email_str),
                name: request.customer_name.clone().map(|name| name.expose().to_string()),
                phone: None,
                billing: None,
            }
        });
        
        // Set payment method details based on the payment method type
        match request.payment_method_data.clone() {
            PaymentMethodData::Card(req_card) => {
                let card = MoneiCard {
                    number: req_card.card_number,
                    expiry_month: req_card.card_exp_month,
                    expiry_year: req_card.card_exp_year,
                    cvc: req_card.card_cvc,
                    cardholder_name: request.customer_name.clone().map(|name| name.expose().to_string()),
                    cardholder_email: request.email.clone().map(|email| email.expose().to_string()),
                };
                
                let payment_method = MoneiPaymentMethod {
                    payment_type: "card".to_string(),
                    card: Some(card),
                };
                
                Ok(Self {
                    amount: item.amount.clone(),
                    currency: request.currency.to_string(),
                    order_id: Some(router_data.connector_request_reference_id.clone()),
                    description: None, // Not available in the request
                    payment_method,
                    customer,
                    transaction_type: if is_auto_capture { "SALE" } else { "AUTH" }.to_string(),
                    complete_url: request.router_return_url.clone(),
                })
            }
            _ => Err(errors::ConnectorError::NotImplemented("Payment method".to_string()).into()),
        }
    }
}
```

### Step 2.4: Update Capture Request Body

**File**: `crates/hyperswitch_connectors/src/connectors/monei.rs`

```rust
// Update the get_request_body method in ConnectorIntegration<Capture, ...> implementation
fn get_request_body(
    &self,
    req: &PaymentsCaptureRouterData,
    _connectors: &Connectors,
) -> CustomResult<RequestContent, errors::ConnectorError> {
    // Convert the amount_to_capture (i64) to a string for the request
    let amount = utils::convert_amount(
        self.amount_converter,
        req.request.amount_to_capture,
        req.request.currency,
    )?;
    
    // Create a simple JSON object with only the amount
    let capture_request = serde_json::json!({
        "amount": amount
    });
    
    Ok(RequestContent::Json(Box::new(capture_request)))
}
```

### Step 2.5: Update Refund Request Structure

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update MoneiRefundRequest struct
#[derive(Default, Debug, Serialize)]
pub struct MoneiRefundRequest {
    /// Amount to refund in minor units (e.g., cents for USD/EUR)
    pub amount: StringMinorUnit,
    /// Reason for the refund (optional)
    #[serde(rename = "refundReason", skip_serializing_if = "Option::is_none")]
    pub refund_reason: Option<String>,
}
```

### Step 2.6: Update TryFrom Implementation for Refund Request

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update TryFrom implementation for refund request
impl<F> TryFrom<&MoneiRouterData<&RefundsRouterData<F>>> for MoneiRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &MoneiRouterData<&RefundsRouterData<F>>) -> Result<Self, Self::Error> {
        let router_data = item.router_data;
        
        Ok(Self {
            amount: item.amount.to_owned(),
            refund_reason: router_data.request.reason.clone(),
        })
    }
}
```

### Step 2.7: Verify Phase 2 Builds Correctly

```bash
cargo build
```

## Phase 3: Status Handling Updates

### Step 3.1: Update MoneiPaymentStatus Enum

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update MoneiPaymentStatus enum to include EXPIRED status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MoneiPaymentStatus {
    SUCCEEDED,
    AUTHORIZED,
    FAILED,
    #[default]
    PENDING,
    CANCELED,
    REFUNDED,
    #[serde(rename = "PARTIALLY_REFUNDED")]
    PartiallyRefunded,
    EXPIRED,
}
```

### Step 3.2: Update Status Mapping Implementation

**File**: `crates/hyperswitch_connectors/src/connectors/monei/transformers.rs`

```rust
// Update the From implementation for status mapping
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
            MoneiPaymentStatus::EXPIRED => Self::AuthorizationFailed,
        }
    }
}
```

### Step 3.3: Verify Phase 3 Builds Correctly

```bash
cargo build
```

## Phase 4: Implement Payment Void

### Step 4.1: Implement ConnectorIntegration for PaymentVoid

**File**: `crates/hyperswitch_connectors/src/connectors/monei.rs`

```rust
// Implement the PaymentVoid trait completely
impl ConnectorIntegration<Void, PaymentsCancelData, PaymentsResponseData> for Monei {
    fn get_headers(
        &self,
        req: &RouterData<Void, PaymentsCancelData, PaymentsResponseData>,
        connectors: &Connectors,
    ) -> CustomResult<Vec<(String, masking::Maskable<String>)>, errors::ConnectorError> {
        self.build_headers(req, connectors)
    }

    fn get_content_type(&self) -> &'static str {
        self.common_get_content_type()
    }

    fn get_url(
        &self,
        req: &RouterData<Void, PaymentsCancelData, PaymentsResponseData>,
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

    fn get_request_body(
        &self,
        req: &RouterData<Void, PaymentsCancelData, PaymentsResponseData>,
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

    fn build_request(
        &self,
        req: &RouterData<Void, PaymentsCancelData, PaymentsResponseData>,
        connectors: &Connectors,
    ) -> CustomResult<Option<Request>, errors::ConnectorError> {
        Ok(Some(
            RequestBuilder::new()
                .method(Method::Post)
                .url(&types::PaymentsVoidType::get_url(self, req, connectors)?)
                .attach_default_headers()
                .headers(types::PaymentsVoidType::get_headers(self, req, connectors)?)
                .set_body(types::PaymentsVoidType::get_request_body(
                    self, req, connectors,
                )?)
                .build(),
        ))
    }

    fn handle_response(
        &self,
        data: &RouterData<Void, PaymentsCancelData, PaymentsResponseData>,
        event_builder: Option<&mut ConnectorEvent>,
        res: Response,
    ) -> CustomResult<RouterData<Void, PaymentsCancelData, PaymentsResponseData>, errors::ConnectorError> {
        let response: monei::MoneiPaymentsResponse = res
            .response
            .parse_struct("Monei PaymentsCancelResponse")
            .change_context(errors::ConnectorError::ResponseDeserializationFailed)?;
        event_builder.map(|i| i.set_response_body(&response));
        router_env::logger::info!(connector_response=?response);
        RouterData::try_from(ResponseRouterData {
            response,
            data: data.clone(),
            http_code: res.status_code,
        })
    }

    fn get_error_response(
        &self,
        res: Response,
        event_builder: Option<&mut ConnectorEvent>,
    ) -> CustomResult<ErrorResponse, errors::ConnectorError> {
        self.build_error_response(res, event_builder)
    }
}
```

### Step 4.2: Verify Phase 4 Builds Correctly

```bash
cargo build
```

## Phase 5: Update Documentation

### Step 5.1: Update the MONEI_specs.md Documentation

**File**: `references/MONEI_specs.md`

Update the authentication method description to match the implementation:

```markdown
### 3.1 Authentication
- **User Story**: As a merchant, I need to authenticate with MONEI API using my API key
- **Implementation Details**:
  - Authentication using API Key in Bearer token format
  - Implementation of `get_auth_header` method to generate appropriate authorization headers
  - Connector auth type: `ConnectorAuthType::HeaderKey` with API key
- **Error Handling**:
  - Handle authentication errors (invalid key, expired token)
  - Map error responses to appropriate Hyperswitch error types
```

### Step 5.2: Update the monei.md Documentation

**File**: `docs/connectors/monei.md`

Update the connector documentation to reflect the changes made.

### Step 5.3: Verify Phase 5 Builds Correctly

```bash
cargo build
```

## Phase 6: Run Tests

### Step 6.1: Run the Connector Tests

```bash
cargo test --package router --test connectors monei -- --nocapture
```

### Step 6.2: Fix Any Test Issues

Update test cases as needed to match the new implementation.

### Step 6.3: Verify Final Build

```bash
cargo build
```

## Conclusion

By following this step-by-step implementation plan, the MONEI connector will be fully aligned with the actual API specifications. Each phase builds on the previous one, with verification steps to ensure the code compiles correctly at each stage.

The implementation addresses:
1. URL structure discrepancies
2. Request/response format mismatches
3. Status mapping gaps
4. Missing void payment functionality
5. Documentation inconsistencies

After all phases are complete, the connector should work correctly with the MONEI payment gateway for all supported operations.
