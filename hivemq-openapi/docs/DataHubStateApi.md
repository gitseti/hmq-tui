# \DataHubStateApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_client_state**](DataHubStateApi.md#get_client_state) | **GET** /api/v1/data-hub/behavior-validation/states/{clientId} | Get the state of a client



## get_client_state

> crate::models::FsmStatesInformationListItem get_client_state(client_id)
Get the state of a client

Use this endpoint to get the stored state of a client for DataHub.  This endpoint requires at least HiveMQ version 4.20.0 on the REST API node.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**client_id** | **String** | The client identifier. | [required] |

### Return type

[**crate::models::FsmStatesInformationListItem**](FsmStatesInformationListItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

