# MONEI Connector Implementation Plan (Updated)

## Overview
This document updates the implementation plan for the MONEI payment gateway integration with Hyperswitch, based on the actual MONEI API documentation. Each flow is analyzed with respect to the actual API endpoints, request/response structures, and requirements.

## Authentication Flow

### API Documentation Details
- **Authentication Method**: API Key in Authorization header with Bearer token format
- **Format**: `Authorization: Bearer YOUR_API_KEY`
- **API Key Types**: Test mode keys (prefixed with `pk_test_`) and live mode keys (prefixed with `pk_live_`)

### Implementation Status
- ✅ Correctly implemented in Step 5-6 with `MoneiAuthType` struct and `get_auth_header` method

## Payment Authorization Flow

### API Documentation Details
- **Endpoint**: `POST https://api.monei.com/v1/payments`
- **Required Fields**:
  - `amount`: Positive integer in smallest currency unit (e.g., cents)
  - `currency`: Three-letter ISO code (e.g., "EUR")
  - `orderId`: Unique identifier from merchant system
- **Optional Fields**:
  - `paymentMethod`: Card details or other payment method information
  - `customer`: Customer information (name, email, phone)
  - `billingDetails`: Billing address information
  - `transactionType`: "SALE" (auto-capture) or "AUTH" (authorization only)
  - Multiple other fields for callbacks, redirects, etc.
- **Response**: Detailed payment object with status, ID, and payment details

### Implementation Status
- ✅ Correctly implemented in Steps 8-11 for request structures and data mapping
- ✅ Correctly implemented in Steps 22-26 for API trait implementations
- ⚠️ Should verify handling of all payment statuses and optional fields

## Payment Sync Flow

### API Documentation Details
- **Endpoint**: `GET https://api.monei.com/v1/payments/:id`
- **Path Parameters**: payment ID
- **Response**: Complete payment object with current status and details

### Implementation Status
- ✅ Correctly implemented in Steps 27-29
- ⚠️ Should verify handling of all payment statuses in response

## Payment Capture Flow

### API Documentation Details
- **Endpoint**: `POST https://api.monei.com/v1/payments/:id/capture`
- **Path Parameters**: payment ID
- **Body Parameters**: 
  - `amount`: Amount to capture (required)
- **Response**: Updated payment object with new status
- **Constraints**: Can only be used with payments in AUTHORIZED status
- **Expiration**: Authorized payments expire after 7 days

### Implementation Status
- ✅ Correctly implemented in Steps 30-33
- ⚠️ Should consider handling of expiration logic

## Payment Cancel Flow (Void)

### API Documentation Details
- **Endpoint**: `POST https://api.monei.com/v1/payments/:id/cancel`
- **Path Parameters**: payment ID
- **Body Parameters**:
  - `cancellationReason`: Reason for cancellation (optional)
- **Response**: Updated payment object with CANCELED status
- **Constraints**: Can only be used with payments in AUTHORIZED status

### Implementation Status
- ❌ Not explicitly included in implementation plan
- ⚠️ Should be implemented as void operation in Hyperswitch

## Refund Flow

### API Documentation Details
- **Endpoint**: `POST https://api.monei.com/v1/payments/:id/refund`
- **Path Parameters**: payment ID
- **Body Parameters**:
  - `amount`: Amount to refund (required)
  - `refundReason`: Reason for refund (optional)
- **Response**: Updated payment object with refund details
- **Constraints**: 
  - Can only refund payments that have been successfully processed
  - Cannot refund more than the original amount
  - Can perform multiple partial refunds

### Implementation Status
- ✅ Correctly implemented in Steps 34-37
- ⚠️ Should verify handling of multiple partial refunds

## Refund Sync Flow

### API Documentation Details
- **Note**: MONEI doesn't have a separate endpoint for refund sync
- **Approach**: Use the payment sync endpoint (`GET /payments/:id`) to check refund status
- **Response Fields**: 
  - `refundedAmount`: Total amount refunded
  - `lastRefundAmount`: Amount refunded in the last transaction
  - `lastRefundReason`: Reason for the last refund
  - `status`: REFUNDED or PARTIALLY_REFUNDED for refunded payments

### Implementation Status
- ✅ Correctly implemented in Steps 38-40
- ⚠️ Should verify proper extraction of refund information from payment response

## Additional Flows in MONEI API

### 1. Confirm Payment Flow
- **Endpoint**: `POST https://api.monei.com/v1/payments/:id/confirm`
- **Description**: Confirms a payment that was created without payment details
- **Usage**: Two-step flow where payment is first created with status PENDING, then confirmed with payment details
- **Implementation Status**: ❌ Not included in implementation plan
- **Recommendation**: Consider if this flow is needed for Hyperswitch integration

### 2. Payment Token Handling
- **Feature**: MONEI supports generating permanent payment tokens for future use
- **Parameter**: `generatePaymentToken: true` when creating or confirming payments
- **Response**: Includes `paymentToken` that can be stored and reused
- **Implementation Status**: ⚠️ Partially handled in request/response structures
- **Recommendation**: Consider enhancing implementation to fully support token storage and reuse

## Status Mapping

### MONEI Payment Statuses
- `SUCCEEDED`: Payment processed successfully with funds captured
- `PENDING`: Payment is being processed and awaiting completion
- `FAILED`: Payment attempt unsuccessful
- `CANCELED`: Payment canceled before completion
- `REFUNDED`: Full payment amount refunded
- `PARTIALLY_REFUNDED`: Portion of payment amount refunded
- `AUTHORIZED`: Payment authorized but funds not captured
- `EXPIRED`: Payment expired without being completed

### Implementation Status
- ✅ Status mapping implemented in Step 13
- ⚠️ Should verify all statuses are handled correctly

## Error Handling

### API Documentation Details
- Response status codes: 200, 400, 401, 404, 422, 500, 503
- Error responses include code, message, and possibly detailed errors array

### Implementation Status
- ✅ Comprehensive error handling implemented in Step 41
- ⚠️ Should verify handling of all possible error codes and messages

## Recommendations for Implementation Updates

1. **Add Payment Cancel (Void) Implementation**:
   - Implement ConnectorIntegration for PaymentVoid
   - Add URL and request/response handling for cancel endpoint

2. **Enhance Error Handling**:
   - Ensure all MONEI-specific error codes are mapped correctly
   - Handle network errors and timeouts appropriately

3. **Improve Status Mapping**:
   - Verify handling of EXPIRED status
   - Ensure proper mapping for all possible state transitions

4. **Consider Additional Features**:
   - Evaluate if Confirm Payment flow is needed
   - Consider supporting payment tokens for recurring payments

5. **Testing Improvements**:
   - Add test cases for cancellation/void
   - Test expiration scenarios
   - Test multiple partial refunds

## Conclusion

The current implementation plan covers most of the core functionality required for MONEI integration. With the recommended updates based on the actual API documentation, the integration will be more robust and feature-complete.
