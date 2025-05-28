# MONEI Integration Documentation

We are integrating **MONEI** into **Hyperswitch** – an open-source payment orchestrator. This documentation provides complete technical details for integrating MONEI’s Payments API with Hyperswitch, focusing exclusively on card payments. It covers API flows, request/response structures, URLs, and authentication details based on the provided MONEI API information.

## Connector URLs

**Urls**
- **baseUrl**: `https://api.monei.com/v1` (Production Base URL for MONEI Payments API)
- **sandboxUrl**: Not explicitly provided in the documentation. MONEI supports a test mode (indicated by `livemode: false` in responses), which uses the same base URL. Sandbox access requires a test API key configured via the MONEI dashboard.
- **Other Important URLs**:
  - **connect_url**: Not applicable. MONEI does not require a specific account connection URL; configuration is done via the MONEI dashboard using an API key.
  - **token_url**: Not applicable. MONEI does not use OAuth2; authentication is handled via API key. Payment tokens for card payments are generated using `monei.js` on the frontend or via the `generatePaymentToken` parameter in the Create Payment API.
  - **documentation_url**: `https://docs.monei.com/` (Official MONEI API documentation)
  - **status_url**: Not explicitly provided. Merchants can monitor service status via the MONEI dashboard or contact MONEI support.
  - **callbackUrl**: Configurable per payment (e.g., `https://example.com/checkout/callback`). Specified in the Create Payment request to receive asynchronous payment status updates via webhook.
  - **completeUrl**: Configurable per payment (e.g., `https://example.com/checkout/complete`). Redirect URL after transaction completion.
  - **failUrl**: Configurable per payment (e.g., `https://example.com/checkout/fail`). Redirect URL for failed transactions.
  - **cancelUrl**: Configurable per payment (e.g., `https://example.com/checkout/cancel`). Redirect URL if the customer cancels the payment.

## Authentication

- **Authentication Type**: API Key (Bearer Token)
- **Steps to Configure**:
  1. Log in to the MONEI dashboard.
  2. Navigate to the API section to generate an API key for production or test mode.
  3. Store the API key securely and include it in the `Authorization` header as a Bearer token for all API requests.
  4. For card payments, use `monei.js` on the frontend to generate temporary `paymentToken`s, ensuring PCI compliance by avoiding direct handling of card details.
- **Example Authentication Headers**:
  ```http
  Authorization: Bearer <API_KEY>
  Content-Type: application/json
  Accept: application/json
  ```
- **Example Curl Command** (for Create Payment):
  ```bash
  curl -L 'https://api.monei.com/v1/payments' \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'Authorization: Bearer <API_KEY>' \
  --data-raw '{...}'
  ```

## Supported Flows with Request/Response Structures

### 1. Authorize / Payment Intent Creation
- **Endpoint URL**: `https://api.monei.com/v1/payments`
- **HTTP Method**: POST
- **Required Headers**:
  ```http
  Content-Type: application/json
  Accept: application/json
  Authorization: Bearer <API_KEY>
  ```
- **Request Payload**:
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
        "cardholderName": "John Doe",
        "cardholderEmail": "john.doe@monei.com"
      }
    },
    "allowedPaymentMethods": ["card"],
    "transactionType": "AUTH",
    "description": "Test Shop - #84370745531439",
    "customer": {
      "email": "john.doe@example.com",
      "name": "John Doe"
    },
    "billingDetails": {
      "name": "John Doe",
      "email": "john.doe@example.com",
      "address": {
        "country": "ES",
        "city": "Málaga",
        "line1": "Fake Street 123",
        "zip": "1234",
        "state": "Málaga"
      }
    },
    "metadata": {
      "systemId": "12345"
    }
  }
  ```
- **Response Payload**:
  ```json
  {
    "id": "af6029f80f5fc73a8ad2753eea0b1be0",
    "amount": 110,
    "currency": "EUR",
    "orderId": "14379133960355",
    "description": "Test Shop - #84370745531439",
    "accountId": "aa9333ba-82de-400c-9ae7-087b9f8d2242",
    "authorizationCode": "475816",
    "livemode": false,
    "status": "AUTHORIZED",
    "statusCode": "E000",
    "statusMessage": "Transaction approved",
    "customer": {
      "email": "john.doe@example.com",
      "name": "John Doe"
    },
    "billingDetails": {
      "name": "John Doe",
      "email": "john.doe@example.com",
      "address": {
        "country": "ES",
        "city": "Málaga",
        "line1": "Fake Street 123",
        "zip": "1234",
        "state": "Málaga"
      }
    },
    "refundedAmount": null,
    "lastRefundAmount": null,
    "lastRefundReason": null,
    "cancellationReason": null,
    "paymentToken": "7cc38b08ff471ccd313ad62b23b9f362b107560b",
    "paymentMethod": {
      "card": {
        "cardholderName": "John Doe",
        "cardholderEmail": "john.doe@monei.com"
      }
    },
    "metadata": {
      "systemId": "12345"
    },
    "createdAt": 1636366897,
    "updatedAt": 1636366897
  }
  ```
- **Curl**:
  ```bash
  curl -L 'https://api.monei.com/v1/payments' \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'Authorization: Bearer <API_KEY>' \
  --data-raw '{
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
        "cardholderName": "John Doe",
        "cardholderEmail": "john.doe@monei.com"
      }
    },
    "allowedPaymentMethods": ["card"],
    "transactionType": "AUTH",
    "description": "Test Shop - #84370745531439",
    "customer": {
      "email": "john.doe@example.com",
      "name": "John Doe"
    },
    "billingDetails": {
      "name": "John Doe",
      "email": "john.doe@example.com",
      "address": {
        "country": "ES",
        "city": "Málaga",
        "line1": "Fake Street 123",
        "zip": "1234",
        "state": "Málaga"
      }
    },
    "metadata": {
      "systemId": "12345"
    }
  }'
  ```

### 2. Capture
- **Endpoint URL**: `https://api.monei.com/v1/payments/:id/capture`
- **HTTP Method**: POST
- **Required Headers**:
  ```http
  Content-Type: application/json
  Accept: application/json
  Authorization: Bearer <API_KEY>
  ```
- **Request Payload**:
  ```json
  {
    "amount": 110
  }
  ```
- **Response Payload**:
  ```json
  {
    "id": "af6029f80f5fc73a8ad2753eea0b1be0",
    "amount": 110,
    "currency": "EUR",
    "orderId": "14379133960355",
    "description": "Test Shop - #84370745531439",
    "accountId": "aa9333ba-82de-400c-9ae7-087b9f8d2242",
    "authorizationCode": "475816",
    "livemode": false,
    "status": "SUCCEEDED",
    "statusCode": "E000",
    "statusMessage": "Transaction approved",
    "customer": {
      "email": "john.doe@example.com",
      "name": "John Doe"
    },
    "billingDetails": {
      "name": "John Doe",
      "email": "john.doe@example.com",
      "address": {
        "country": "ES",
        "city": "Málaga",
        "line1": "Fake Street 123",
        "zip": "1234",
        "state": "Málaga"
      }
    },
    "refundedAmount": null,
    "lastRefundAmount": null,
    "lastRefundReason": null,
    "cancellationReason": null,
    "paymentToken": "7cc38b08ff471ccd313ad62b23b9f362b107560b",
    "paymentMethod": {
      "card": {
        "cardholderName": "John Doe",
        "cardholderEmail": "john.doe@monei.com"
      }
    },
    "metadata": {
      "systemId": "12345"
    },
    "createdAt": 1636366897,
    "updatedAt": 1636366897
  }
  ```
- **Curl**:
  ```bash
  curl -L 'https://api.monei.com/v1/payments/af6029f80f5fc73a8ad2753eea0b1be0/capture' \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'Authorization: Bearer <API_KEY>' \
  -d '{
    "amount": 110
  }'
  ```

### 3. Refund
- **Endpoint URL**: `https://api.monei.com/v1/payments/:id/refund`
- **HTTP Method**: POST
- **Required Headers**:
  ```http
  Content-Type: application/json
  Accept: application/json
  Authorization: Bearer <API_KEY>
  ```
- **Request Payload**:
  ```json
  {
    "amount": 110,
    "refundReason": "requested_by_customer"
  }
  ```
- **Response Payload**:
  ```json
  {
    "id": "af6029f80f5fc73a8ad2753eea0b1be0",
    "amount": 110,
    "currency": "EUR",
    "orderId": "14379133960355",
    "description": "Test Shop - #84370745531439",
    "accountId": "aa9333ba-82de-400c-9ae7-087b9f8d2242",
    "authorizationCode": "475816",
    "livemode": false,
    "status": "REFUNDED",
    "statusCode": "E000",
    "statusMessage": "Transaction approved",
    "customer": {
      "email": "john.doe@example.com",
      "name": "John Doe"
    },
    "billingDetails": {
      "name": "John Doe",
      "email": "john.doe@example.com",
      "address": {
        "country": "ES",
        "city": "Málaga",
        "line1": "Fake Street 123",
        "zip": "1234",
        "state": "Málaga"
      }
    },
    "refundedAmount": 110,
    "lastRefundAmount": 110,
    "lastRefundReason": "requested_by_customer",
    "cancellationReason": null,
    "paymentToken": "7cc38b08ff471ccd313ad62b23b9f362b107560b",
    "paymentMethod": {
      "card": {
        "cardholderName": "John Doe",
        "cardholderEmail": "john.doe@monei.com"
      }
    },
    "metadata": {
      "systemId": "12345"
    },
    "createdAt": 1636366897,
    "updatedAt": 1636366897
  }
  ```
- **Curl**:
  ```bash
  curl -L 'https://api.monei.com/v1/payments/af6029f80f5fc73a8ad2753eea0b1be0/refund' \
  -H 'Content-Type: application/json' \
  -H 'Accept: application/json' \
  -H 'Authorization: Bearer <API_KEY>' \
  -d '{
    "amount": 110,
    "refundReason": "requested_by_customer"
  }'
  ```
**Remaining in MONEI Integration Documentation 2.markdown [Read it]**