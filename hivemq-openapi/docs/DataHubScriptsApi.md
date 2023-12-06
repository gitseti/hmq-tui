# \DataHubScriptsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_script**](DataHubScriptsApi.md#create_script) | **POST** /api/v1/data-hub/scripts | Create a new script
[**delete_script**](DataHubScriptsApi.md#delete_script) | **DELETE** /api/v1/data-hub/scripts/{scriptId} | Delete a script
[**get_all_scripts**](DataHubScriptsApi.md#get_all_scripts) | **GET** /api/v1/data-hub/scripts | Get all scripts
[**get_script**](DataHubScriptsApi.md#get_script) | **GET** /api/v1/data-hub/scripts/{scriptId} | Get a script



## create_script

> crate::models::Script create_script(script)
Create a new script

Creates a script

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**script** | [**Script**](Script.md) | The script that should be created. | [required] |

### Return type

[**crate::models::Script**](Script.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_script

> delete_script(script_id)
Delete a script

Deletes the selected script.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**script_id** | **String** | The script identifier of the script to delete. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_scripts

> crate::models::ScriptList get_all_scripts(fields, function_types, script_ids, limit, cursor)
Get all scripts

Get all scripts.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, version, description, runtime, functionType, createdAt |  |
**function_types** | Option<**String**> | Comma-separated list of function types used for filtering. Multiple filters can be applied together. |  |
**script_ids** | Option<**String**> | Comma-separated list of script ids used for filtering. Multiple filters can be applied together. |  |
**limit** | Option<**i32**> | Specifies the page size for the returned results. Has to be between 10 and 500. Default page size is 50. |  |
**cursor** | Option<**String**> | The cursor that has been returned by the previous result page. Do not pass this parameter if you want to fetch the first page. |  |

### Return type

[**crate::models::ScriptList**](ScriptList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_script

> crate::models::Script get_script(script_id, fields)
Get a script

Get a specific script.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**script_id** | **String** | The identifier of the script. | [required] |
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, version, description, runtime, functionType, createdAt |  |

### Return type

[**crate::models::Script**](Script.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

