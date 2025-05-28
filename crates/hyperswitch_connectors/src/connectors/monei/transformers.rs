use common_enums::enums;
use common_utils::types::StringMinorUnit;
use hyperswitch_domain_models::{
    payment_method_data::PaymentMethodData,
    router_data::{ConnectorAuthType, RouterData},
    router_flow_types::refunds::{Execute, RSync},
    router_request_types::ResponseId,
    router_response_types::{PaymentsResponseData, RefundsResponseData},
    types::{PaymentsAuthorizeRouterData, RefundsRouterData},
};
use hyperswitch_interfaces::errors;
use masking::{ExposeInterface, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    types::{RefundsResponseRouterData, ResponseRouterData},
    utils::PaymentsAuthorizeRequestData,
};

// MoneiRouterData is a wrapper struct that holds both the amount and router data
// This allows us to handle amount conversion along with the router data in one place
pub struct MoneiRouterData<T> {
    pub amount: StringMinorUnit, // Amount in the format accepted by MONEI (minor units as a string)
    pub router_data: T,          // The router data containing payment/refund information
}

impl<T> From<(StringMinorUnit, T)> for MoneiRouterData<T> {
    fn from((amount, item): (StringMinorUnit, T)) -> Self {
        // The amount is already converted to the required format (StringMinorUnit)
        // by the connector's amount_converter before being passed here
        Self {
            amount,
            router_data: item,
        }
    }
}

#[derive(Default, Debug, Serialize, PartialEq)]
pub struct MoneiPaymentsRequest {
    /// The payment amount in minor units (e.g. cents for USD)
    amount: StringMinorUnit,
    /// Currency code in ISO 4217 format
    currency: String,
    /// A unique identifier for the payment
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    /// Description of the payment shown to the customer
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Payment method details (card for card payments)
    payment_method: MoneiPaymentMethod,
    /// Customer information
    #[serde(skip_serializing_if = "Option::is_none")]
    customer: Option<MoneiCustomer>,
    /// Whether to capture the payment immediately (true) or authorize only (false)
    complete: bool,
    /// Return URL where the customer will be redirected after the payment
    #[serde(skip_serializing_if = "Option::is_none")]
    return_url: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneiPaymentMethod {
    /// Type of payment method (card, paypal, etc.)
    #[serde(rename = "type")]
    payment_type: String,
    /// Card details (required for card payments)
    #[serde(skip_serializing_if = "Option::is_none")]
    card: Option<MoneiCard>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MoneiCard {
    /// Card number
    number: cards::CardNumber,
    /// Expiry month (MM format)
    expiry_month: Secret<String>,
    /// Expiry year (YY format)
    expiry_year: Secret<String>,
    /// Card security code
    cvc: Secret<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneiCustomer {
    /// Customer's email address
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<String>,
    /// Customer's name
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Customer's phone
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<String>,
    /// Billing details
    #[serde(skip_serializing_if = "Option::is_none")]
    billing: Option<MoneiBillingDetails>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneiBillingDetails {
    /// Billing address
    address: MoneiAddress,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneiAddress {
    /// First line of the address
    #[serde(skip_serializing_if = "Option::is_none")]
    line1: Option<String>,
    /// Second line of the address
    #[serde(skip_serializing_if = "Option::is_none")]
    line2: Option<String>,
    /// City
    #[serde(skip_serializing_if = "Option::is_none")]
    city: Option<String>,
    /// State/province/region
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    /// Postal/ZIP code
    #[serde(skip_serializing_if = "Option::is_none")]
    postal_code: Option<String>,
    /// Two-letter country code (ISO 3166-1 alpha-2)
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
}

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
                };
                
                let payment_method = MoneiPaymentMethod {
                    payment_type: "card".to_string(),
                    card: Some(card),
                };
                
                Ok(Self {
                    amount: item.amount.clone(),
                    currency: request.currency.to_string(),
                    reference: Some(router_data.connector_request_reference_id.clone()),
                    description: None, // Not available in the request
                    payment_method,
                    customer,
                    complete: is_auto_capture,
                    return_url: request.router_return_url.clone(),
                })
            }
            _ => Err(errors::ConnectorError::NotImplemented("Payment method".to_string()).into()),
        }
    }
}

//TODO: Fill the struct with respective fields
// Auth Struct for MONEI Bearer token authentication
pub struct MoneiAuthType {
    pub(super) api_key: Secret<String>,
}

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
// PaymentsResponse
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
}

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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MoneiPaymentsResponse {
    /// Unique payment identifier
    id: String,
    /// Payment amount in minor units (e.g., cents for USD/EUR)
    amount: i64,
    /// Currency code in ISO 4217 format
    currency: String,
    /// Merchant's order reference
    #[serde(rename = "orderId")]
    order_id: Option<String>,
    /// Payment description
    description: Option<String>,
    /// MONEI account ID
    #[serde(rename = "accountId")]
    account_id: String,
    /// Authorization code from the payment processor
    #[serde(rename = "authorizationCode")]
    authorization_code: Option<String>,
    /// Whether the payment is in live mode (true) or test mode (false)
    livemode: bool,
    /// Payment status
    status: MoneiPaymentStatus,
    /// Status code from the payment processor
    #[serde(rename = "statusCode")]
    status_code: String,
    /// Human-readable status message
    #[serde(rename = "statusMessage")]
    status_message: String,
    /// Customer information
    customer: Option<MoneiCustomer>,
    /// Billing details
    #[serde(rename = "billingDetails")]
    billing_details: Option<MoneiBillingDetails>,
    /// Total amount refunded
    #[serde(rename = "refundedAmount")]
    refunded_amount: Option<i64>,
    /// Amount of the last refund
    #[serde(rename = "lastRefundAmount")]
    last_refund_amount: Option<i64>,
    /// Reason for the last refund
    #[serde(rename = "lastRefundReason")]
    last_refund_reason: Option<String>,
    /// Reason for cancellation if the payment was canceled
    #[serde(rename = "cancellationReason")]
    cancellation_reason: Option<String>,
    /// Payment token for future use
    #[serde(rename = "paymentToken")]
    payment_token: Option<String>,
    /// Payment method details
    #[serde(rename = "paymentMethod")]
    payment_method: Option<MoneiPaymentMethod>,
    /// Additional custom data
    metadata: Option<serde_json::Value>,
    /// Unix timestamp when the payment was created
    #[serde(rename = "createdAt")]
    created_at: i64,
    /// Unix timestamp when the payment was last updated
    #[serde(rename = "updatedAt")]
    updated_at: i64,
}

impl<F, T> TryFrom<ResponseRouterData<F, MoneiPaymentsResponse, T, PaymentsResponseData>>
    for RouterData<F, T, PaymentsResponseData>
{
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: ResponseRouterData<F, MoneiPaymentsResponse, T, PaymentsResponseData>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            status: common_enums::AttemptStatus::from(item.response.status),
            response: Ok(PaymentsResponseData::TransactionResponse {
                resource_id: ResponseId::ConnectorTransactionId(item.response.id),
                redirection_data: Box::new(None),
                mandate_reference: Box::new(None),
                connector_metadata: None,
                network_txn_id: None,
                connector_response_reference_id: None,
                incremental_authorization_allowed: None,
                charges: None,
            }),
            ..item.data
        })
    }
}

// REFUND :
// Type definition for RefundRequest
#[derive(Default, Debug, Serialize)]
pub struct MoneiRefundRequest {
    /// Payment ID to be refunded
    pub payment: String,
    /// Amount to refund in minor units (e.g., cents for USD/EUR)
    pub amount: StringMinorUnit,
    /// Reason for the refund (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Reference for merchant's tracking (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

impl<F> TryFrom<&MoneiRouterData<&RefundsRouterData<F>>> for MoneiRefundRequest {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(item: &MoneiRouterData<&RefundsRouterData<F>>) -> Result<Self, Self::Error> {
        let router_data = item.router_data;
        
        // Extract the payment ID from the connector_transaction_id field
        let payment_id = router_data
            .request
            .connector_transaction_id
            .clone();
        
        Ok(Self {
            payment: payment_id,
            amount: item.amount.to_owned(),
            reason: router_data.request.reason.clone(),
            reference: Some(router_data.connector_request_reference_id.clone()),
        })
    }
}

// Type definition for Refund Response

#[derive(Debug, Serialize, Default, Deserialize, Clone)]
#[serde(rename_all = "UPPERCASE")]
pub enum MoneiRefundStatus {
    #[default]
    PENDING,
    SUCCEEDED,
    FAILED,
    CANCELED,
}

impl From<MoneiRefundStatus> for enums::RefundStatus {
    fn from(item: MoneiRefundStatus) -> Self {
        match item {
            MoneiRefundStatus::SUCCEEDED => Self::Success,
            MoneiRefundStatus::FAILED => Self::Failure,
            MoneiRefundStatus::PENDING => Self::Pending,
            MoneiRefundStatus::CANCELED => Self::Failure,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MoneiRefundResponse {
    /// Unique refund identifier
    id: String,
    /// Payment ID that was refunded
    #[serde(rename = "paymentId")]
    payment_id: String,
    /// Refund amount in minor units (e.g., cents for USD/EUR)
    amount: i64,
    /// Currency code in ISO 4217 format
    currency: String,
    /// Refund status
    status: MoneiRefundStatus,
    /// Status code from the payment processor
    #[serde(rename = "statusCode")]
    status_code: Option<String>,
    /// Human-readable status message
    #[serde(rename = "statusMessage")]
    status_message: Option<String>,
    /// Reason for the refund
    reason: Option<String>,
    /// Merchant reference for the refund
    reference: Option<String>,
    /// Whether the refund is in live mode (true) or test mode (false)
    livemode: bool,
    /// Unix timestamp when the refund was created
    #[serde(rename = "createdAt")]
    created_at: i64,
    /// Unix timestamp when the refund was last updated
    #[serde(rename = "updatedAt")]
    updated_at: i64,
}

impl TryFrom<RefundsResponseRouterData<Execute, MoneiRefundResponse>> for RefundsRouterData<Execute> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<Execute, MoneiRefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

impl TryFrom<RefundsResponseRouterData<RSync, MoneiRefundResponse>> for RefundsRouterData<RSync> {
    type Error = error_stack::Report<errors::ConnectorError>;
    fn try_from(
        item: RefundsResponseRouterData<RSync, MoneiRefundResponse>,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            response: Ok(RefundsResponseData {
                connector_refund_id: item.response.id.to_string(),
                refund_status: enums::RefundStatus::from(item.response.status),
            }),
            ..item.data
        })
    }
}

// Error Response Structure
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct MoneiErrorDetail {
    /// Specific parameter that caused the error
    pub param: Option<String>,
    /// Location of the parameter in the request
    pub location: Option<String>,
    /// Detailed description of the error
    pub message: String,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct MoneiErrorResponse {
    /// HTTP status code
    pub status_code: u16,
    /// Error type code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Optional reason for the error
    pub reason: Option<String>,
    /// Detailed error information for validation errors
    #[serde(rename = "details")]
    pub error_details: Option<Vec<MoneiErrorDetail>>,
    /// Request ID for support reference
    #[serde(rename = "requestId")]
    pub request_id: Option<String>,
}
