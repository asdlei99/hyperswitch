# Get Payment

**GET** `https://api.monei.com/v1/payments/:id`

Retrieves the complete details of an existing payment by its unique ID.

This endpoint returns all available information about the payment, including its current status, amount, customer details, timestamps, and transaction history. Use this to check the status of a payment, verify payment details, or retrieve information for your records.

Supply the unique payment ID that was returned from your previous request.

## Request

### Path Parameters

| Parameter | Type | Description | Required |
|-----------|------|-------------|----------|
| **id** | Payment-Id (string) | The payment ID | Yes |

## Responses

| Status Code | Description |
|-------------|-------------|
| 200 | A payment object |
| 400 | Bad Request |
| 401 | Unauthorized |
| 404 | Not Found |
| 422 | Unprocessable Entity |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

### Response Schema (200)

**Content-Type**: `application/json`

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| **id** | Payment-Id (string, required) | Unique identifier for the payment. | `af6029f80f5fc73a8ad2753eea0b1be0` |
| **amount** | int32 (required) | Amount intended to be collected by this payment. A positive integer representing how much to charge in the smallest currency unit (e.g., 100 cents to charge 1.00 USD). | `110` |
| **currency** | Payment-Currency (string, required) | Three-letter ISO currency code, in uppercase. Must be a supported currency. | `EUR` |
| **orderId** | Payment-OrderId (string) | An order ID from your system. A unique identifier that can be used to reconcile the payment with your internal system. | `14379133960355` |
| **description** | Payment-Description (string) | An arbitrary string attached to the payment. Often useful for displaying to users. | `Test Shop - #84370745531439` |
| **accountId** | AccountId (string, required) | MONEI Account identifier. | `aa9333ba-82de-400c-9ae7-087b9f8d2242` |
| **authorizationCode** | Payment-AuthorizationCode (string) | Unique identifier provided by the bank performing transaction. | `475816` |
| **livemode** | Livemode (boolean, required) | Has the value `true` if the resource exists in live mode or the value `false` if the resource exists in test mode. | `false` |
| **status** | Payment-Status (string, required) | The status of the payment. | `PENDING` |
| | | Enum Values: <br> `SUCCEEDED` (The payment has been successfully processed and funds have been captured), <br> `PENDING` (The payment is being processed and awaiting completion), <br> `FAILED` (The payment attempt was unsuccessful), <br> `CANCELED` (The payment was canceled before completion), <br> `REFUNDED` (The full payment amount has been refunded), <br> `PARTIALLY_REFUNDED` (Only a portion of the payment amount has been refunded), <br> `AUTHORIZED` (The payment has been authorized but funds have not been captured yet), <br> `EXPIRED` (The payment has expired without being completed) | |
| **statusCode** | Payment-StatusCode (string) | Payment status code. | `E000` |
| **statusMessage** | Payment-StatusMessage (string) | Human-readable status message, can be displayed to a user. | `Transaction approved` |
| **customer** | object | | |
| **shop** | object | | |
| **billingDetails** | object | | |
| **shippingDetails** | object | | |
| **refundedAmount** | int32 | Amount in cents refunded (can be less than the amount attribute on the payment if a partial refund was issued). | `null` |
| **lastRefundAmount** | int32 | Amount in cents refunded in the last transaction. | `null` |
| **lastRefundReason** | Payment-LastRefundReason (string) | The reason of the last refund transaction. <br> Possible values: `duplicated`, `fraudulent`, `requested_by_customer` | `null` |
| **cancellationReason** | Payment-CancellationReason (string) | The reason for canceling the Payment. <br> Possible values: `duplicated`, `fraudulent`, `requested_by_customer` | `null` |
| **sessionDetails** | Payment-SessionDetails | | |
| **traceDetails** | Payment-TraceDetails | | |
| **paymentToken** | Payment-PaymentToken (string) | A permanent token represents a payment method used in the payment. Pass `generatePaymentToken: true` when you create a payment to generate it. You can pass it as `paymentToken` parameter to create other payments with the same payment method. This token does not expire, and should only be used server-side. | `7cc38b08ff471ccd313ad62b23b9f362b107560b` |
| **paymentMethod** | object | | |
| **sequence** | object | | |
| **sequenceId** | Payment-SequenceId (string) | A permanent identifier that refers to the initial payment of a sequence of payments. This value needs to be sent in the path for RECURRING payments. | `62b23b9f3627cc38b08ff471ccd313ad` |
| **storeId** | Payment-StoreId (string) | A unique identifier of the Store. If specified, the payment is attached to this Store. | `e5f28150d9e8974c58ab5ec9c4a880f8734dcf05` |
| **pointOfSaleId** | Payment-PointOfSaleId (string) | A unique identifier of the Point of Sale. If specified, the payment is attached to this Point of Sale. If there is a QR code attached to the same Point of Sale, this payment will be available by scanning the QR code. | `fb269cccfa0cc021f5d0b8eb1421646c696213e1` |
| **metadata** | object | A set of key-value pairs that you can attach to a resource. This can be useful for storing additional information about the resource in a structured format. | `{"systemId":"12345"}` |
| **nextAction** | object | | |
| **createdAt** | int64 | Time at which the resource was created. Measured in seconds since the Unix epoch. | `1636366897` |
| **updatedAt** | int64 | Time at which the resource was updated last time. Measured in seconds since the Unix epoch. | `1636366897` |

### Authorization

| Name | Type | In | Description |
|------|------|----|-------------|
| **Authorization** | apiKey | header | The MONEI API uses API keys to authenticate requests. Each request must include your API key in the Authorization header. You can view and manage your API keys in the MONEI Dashboard. MONEI provides two types of API keys: test mode keys (prefixed with `pk_test_`) and live mode keys (prefixed with `pk_live_`). |

## Example Request

```bash
curl -L 'https://api.monei.com/v1/payments/:id' \
-H 'Accept: application/json' \
-H 'Authorization: <Authorization>'
```