# MONEI Connector Documentation

## Overview

MONEI is a payment gateway that enables businesses to accept card payments. The Hyperswitch MONEI connector supports the following payment operations:

- Payment Authorization (AUTH)
- Payment Capture
- Payment Synchronization
- Refund Processing
- Refund Synchronization

## Authentication

MONEI uses API Key authentication with a Bearer token.

| Authentication Parameter | Description |
|--------------------------|-------------|
| API Key | Used as a Bearer token in the Authorization header |

## Supported Payment Methods

| Payment Method | Support Level |
|----------------|--------------|
| Cards          | Full support |

## Supported Currencies

MONEI supports standard currencies including USD, EUR, GBP, and others. Amounts are processed in minor units (cents).

## Payment Flows

### Authorization Flow

The connector supports both authorization-only (AUTH) and direct capture (SALE) transactions:
- When `capture_method` is set to `automatic`, the connector performs a direct SALE transaction
- When `capture_method` is set to `manual`, the connector performs an AUTH transaction

### Capture Flow

For payments authorized with `manual` capture method, the connector supports:
- Full capture
- Partial capture

### Refund Flow

The connector supports:
- Full refunds
- Partial refunds

## Configuration Parameters

| Parameter | Required | Description |
|-----------|----------|-------------|
| API Key | Yes | MONEI API Key used for authentication |
| Base URL | Yes | API endpoint for MONEI (defaults to `https://api.monei.com/v1`) |

## Error Handling

The connector maps MONEI error codes to appropriate Hyperswitch error types and provides detailed error messages.

## Webhook Support

Webhook support is not currently implemented for this connector.

## Limitations

- 3D Secure processing is not currently supported
- Webhook handling is not implemented

## Testing

The connector includes comprehensive test cases that cover:
- Successful payment authorization
- Payment captures (full and partial)
- Payment synchronization
- Refund processing (full and partial)
- Error scenarios

## Connector Information

- Integration Date: May 2025
- Connector Type: Payment Processor
- API Version: v1
