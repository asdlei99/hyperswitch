# MONEI Connector Technical Specification

## 1. System Overview

### Core Purpose
Integrate MONEI payment gateway with Hyperswitch to enable card payment processing through MONEI's API.

### Key Workflows
1. **Payment Authorization**: Create payment intents for card payments
2. **Payment Capture**: Capture previously authorized payments
3. **Payment Sync**: Check payment status
4. **Refund Processing**: Process refunds for completed payments
5. **Refund Sync**: Check refund status

### System Architecture
The MONEI connector will follow the Hyperswitch connector architecture pattern, consisting of:
- Main connector module (`monei.rs`)
- Transformers module for data mapping (`transformers.rs`)
- Test module for integration tests

## 2. Project Structure

```
hyperswitch_connectors/src/connectors/
├── monei
│   └── transformers.rs
└── monei.rs

router/tests/connectors/
└── monei.rs
```

## 3. Feature Specification

### 3.1 Authentication
- **User Story**: As a merchant, I need to authenticate with MONEI API using my API key
- **Implementation Details**:
  - Authentication using API Key in Bearer token format
  - Implementation of `get_auth_header` method to generate appropriate authorization headers
  - Connector auth type: `ConnectorAuthType::BodyKey` with API key
- **Error Handling**:
  - Handle authentication errors (invalid key, expired token)
  - Map error responses to appropriate Hyperswitch error types

### 3.2 Payment Authorization
- **User Story**: As a merchant, I need to process card payment authorizations through MONEI
- **Implementation Details**:
  - Endpoint: `POST https://api.monei.com/v1/payments`
  - Request transformation: Map Hyperswitch payment data to MONEI API format
  - Include customer data, billing details, and payment method information
  - Support for transaction type: `AUTH` for authorization-only payments
  - Support for direct capture using `transactionType: "SALE"`
- **Error Handling**:
  - Handle card validation errors
  - Map MONEI status codes and error messages to Hyperswitch format

### 3.3 Payment Capture
- **User Story**: As a merchant, I need to capture previously authorized payments
- **Implementation Details**:
  - Endpoint: `POST https://api.monei.com/v1/payments/:id/capture`
  - Support for full and partial captures
  - Map payment ID and amount correctly
- **Error Handling**:
  - Handle capture-specific errors (payment already captured, expired authorization)

### 3.4 Payment Sync
- **User Story**: As a merchant, I need to check the status of payments
- **Implementation Details**:
  - Endpoint: `GET https://api.monei.com/v1/payments/:id`
  - Map MONEI payment status to Hyperswitch payment status
- **Error Handling**:
  - Handle cases where payment is not found
  - Process status updates for pending, succeeded, and failed payments

### 3.5 Refund Processing
- **User Story**: As a merchant, I need to process refunds for completed payments
- **Implementation Details**:
  - Endpoint: `POST https://api.monei.com/v1/payments/:id/refund`
  - Support for full and partial refunds
  - Include refund reason when available
- **Error Handling**:
  - Handle refund-specific errors (insufficient funds, payment not refundable)

### 3.6 Refund Sync
- **User Story**: As a merchant, I need to check the status of refunds
- **Implementation Details**:
  - Use the payment sync endpoint to check refund status
  - Map refunded amount and status from MONEI response
- **Error Handling**:
  - Handle cases where refund information is not available

## 4. Database Schema
No direct database modifications required. The connector will use existing Hyperswitch database structures.

## 5. Connector Implementation

### 5.1 Constant Definitions
```rust
pub const BASE_URL: &str = "https://api.monei.com/v1";
pub const PAYMENTS_URL: &str = "/payments";
pub const CAPTURES_URL: &str = "/payments/:id/capture";
pub const REFUNDS_URL: &str = "/payments/:id/refund";
```

### 5.2 Struct Definitions

#### MONEI Main Connector
```rust
#[derive(Clone)]
pub struct Monei {
    amount_converter: &'static (dyn AmountConvertor<Output = StringMinorUnit> + Sync),
}

impl Monei {
    pub fn new() -> &'static Self {
        &Self {
            amount_converter: &StringMinorUnitForConnector,
        }
    }
}
```

#### Auth Type
```rust
pub struct MoneiAuthType {
    pub(super) api_key: Secret<String>,
}

impl TryFrom<&ConnectorAuthType> for MoneiAuthType {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(auth_type: &ConnectorAuthType) -> Result<Self, Self::Error> {
        match auth_type {
            ConnectorAuthType::BodyKey { api_key, .. } => Ok(Self {
                api_key: api_key.to_owned(),
            }),
            _ => Err(errors::ConnectorError::FailedToObtainAuthType.into()),
        }
    }
}
```

### 5.3 Request Types

#### Payments Request
```rust
#[derive(Default, Debug, Serialize)]
pub struct MoneiPaymentsRequest {
    amount: i64,
    currency: String,
    orderId: String,
    callbackUrl: Option<String>,
    completeUrl: Option<String>,
    failUrl: Option<String>,
    cancelUrl: Option<String>,
    paymentToken: Option<String>,
    sessionId: Option<String>,
    generatePaymentToken: bool,
    paymentMethod: MoneiPaymentMethod,
    allowedPaymentMethods: Vec<String>,
    transactionType: String,
    description: Option<String>,
    customer: Option<MoneiCustomer>,
    billingDetails: Option<MoneiBillingDetails>,
    metadata: Option<serde_json::Value>,
}

#[derive(Default, Debug, Serialize)]
pub struct MoneiPaymentMethod {
    card: MoneiCard,
}

#[derive(Default, Debug, Serialize)]
pub struct MoneiCard {
    cardholderName: Option<String>,
    cardholderEmail: Option<String>,
}

#[derive(Default, Debug, Serialize)]
pub struct MoneiCustomer {
    email: Option<String>,
    name: Option<String>,
}

#[derive(Default, Debug, Serialize)]
pub struct MoneiBillingDetails {
    name: Option<String>,
    email: Option<String>,
    address: Option<MoneiAddress>,
}

#[derive(Default, Debug, Serialize)]
pub struct MoneiAddress {
    country: Option<String>,
    city: Option<String>,
    line1: Option<String>,
    zip: Option<String>,
    state: Option<String>,
}
```

#### Capture Request
```rust
#[derive(Default, Debug, Serialize)]
pub struct MoneiCaptureRequest {
    amount: i64,
}
```

#### Refund Request
```rust
#[derive(Default, Debug, Serialize)]
pub struct MoneiRefundRequest {
    amount: i64,
    refundReason: Option<String>,
}
```

### 5.4 Response Types

#### Payments Response
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MoneiPaymentsResponse {
    id: String,
    amount: i64,
    currency: String,
    orderId: String,
    description: Option<String>,
    accountId: String,
    authorizationCode: Option<String>,
    livemode: bool,
    status: MoneiPaymentStatus,
    statusCode: String,
    statusMessage: String,
    customer: Option<MoneiCustomer>,
    billingDetails: Option<MoneiBillingDetails>,
    refundedAmount: Option<i64>,
    lastRefundAmount: Option<i64>,
    lastRefundReason: Option<String>,
    cancellationReason: Option<String>,
    paymentToken: Option<String>,
    paymentMethod: Option<MoneiPaymentMethod>,
    metadata: Option<serde_json::Value>,
    createdAt: i64,
    updatedAt: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MoneiPaymentStatus {
    SUCCEEDED,
    AUTHORIZED,
    FAILED,
    PENDING,
    CANCELED,
    REFUNDED,
    PARTIALLY_REFUNDED,
}
```

#### Error Response
```rust
#[derive(Debug, Clone, Deserialize)]
pub struct MoneiErrorResponse {
    code: String,
    message: String,
    errors: Option<Vec<MoneiErrorDetail>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoneiErrorDetail {
    code: String,
    message: String,
    path: Option<String>,
}
```

### 5.5 Status Mapping

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
            MoneiPaymentStatus::PARTIALLY_REFUNDED => Self::Charged,
        }
    }
}
```

## 6. Implementation Strategy

### 6.1 API Trait Implementations

Implement the following API traits for the MONEI connector:
- `Payment`
- `PaymentAuthorize`
- `PaymentSync`
- `PaymentCapture`
- `Refund`
- `RefundExecute`
- `RefundSync`

### 6.2 Connector Integration Implementations

For each flow, implement the `ConnectorIntegration` trait with appropriate methods:
- `get_headers`
- `get_content_type`
- `get_url`
- `get_request_body`
- `build_request`
- `handle_response`
- `get_error_response`

### 6.3 Data Transformation

Implement transformers for:
- Converting Hyperswitch payment data to MONEI format
- Converting MONEI responses to Hyperswitch format
- Mapping error responses
- Converting amount formats

### 6.4 Connector Validation

Implement the `ConnectorValidation` trait to validate connector parameters.

## 7. Testing Strategy

### 7.1 Unit Tests
- Test all transformers
- Test status mapping
- Test error response handling

### 7.2 Integration Tests
Implement the following test cases in `monei.rs` under the `router/tests/connectors` directory:
- Test successful payment authorization
- Test payment authorization with failure
- Test payment capture
- Test payment sync
- Test refund execution
- Test refund sync

### 7.3 Error Scenarios to Test
- Invalid authentication
- Card declined
- Insufficient funds
- Invalid card details
- Expired card
- Card not supported
- 3D Secure required
- Payment not found
- Refund not allowed
- Duplicate transaction

## 8. Implementation Plan

### Phase 1: Core Structure Setup
1. Create base connector files (`monei.rs` and `transformers.rs`)
2. Implement constants and basic structure
3. Implement authentication

### Phase 2: Payment Flows
1. Implement payment authorization
2. Implement payment capture
3. Implement payment sync

### Phase 3: Refund Flows
1. Implement refund execution
2. Implement refund sync

### Phase 4: Error Handling
1. Implement comprehensive error handling
2. Map all error codes to appropriate Hyperswitch error types

### Phase 5: Testing
1. Implement unit tests
2. Implement integration tests
3. Test with sandbox environment

## 9. Implementation Notes and Best Practices

1. **Amount Handling**
   - MONEI processes amounts in minor units (cents for USD/EUR)
   - Use the connector's amount converter for accurate conversion

2. **Authentication**
   - Always send the API key as a Bearer token in the Authorization header
   - Securely store API keys

3. **Error Handling**
   - Map MONEI status codes to Hyperswitch attempt statuses
   - Provide detailed error messages for troubleshooting

4. **Metadata Usage**
   - Use metadata field for storing Hyperswitch-specific information
   - Include necessary transaction references

5. **Code Consistency**
   - Follow Hyperswitch coding patterns and standards
   - Reuse utility functions for common operations
   - Maintain consistent naming conventions

## 10. Documentation Requirements

Update the following documentation:
1. Add MONEI to supported connectors list
2. Document MONEI-specific configuration parameters
3. Provide sample API key format
4. Document supported payment methods and currencies
