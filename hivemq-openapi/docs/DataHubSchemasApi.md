# \DataHubSchemasApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_schema**](DataHubSchemasApi.md#create_schema) | **POST** /api/v1/data-hub/schemas | Create a new schema
[**delete_schema**](DataHubSchemasApi.md#delete_schema) | **DELETE** /api/v1/data-hub/schemas/{schemaId} | Delete all versions of the schema
[**get_all_schemas**](DataHubSchemasApi.md#get_all_schemas) | **GET** /api/v1/data-hub/schemas | Get all schemas
[**get_schema**](DataHubSchemasApi.md#get_schema) | **GET** /api/v1/data-hub/schemas/{schemaId} | Get a schema



## create_schema

> crate::models::Schema create_schema(schema)
Create a new schema

Creates a schema  This endpoint requires at least HiveMQ version 4.15.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schema** | [**Schema**](Schema.md) | The schema that should be created. | [required] |

### Return type

[**crate::models::Schema**](Schema.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_schema

> delete_schema(schema_id)
Delete all versions of the schema

Deletes the selected schema and all associated versions of the schema.    This endpoint requires HiveMQ version4.15.0 or above on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schema_id** | **String** | The schema identifier of the schema versions to delete. | [required] |

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_schemas

> crate::models::SchemaList get_all_schemas(fields, types, schema_ids, limit, cursor)
Get all schemas

Get all schemas.    This endpoint returns the content of the schemas with the content-type `application/json`.    This endpoint requires at least HiveMQ version 4.16.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, type, schemaDefinition, createdAt |  |
**types** | Option<**String**> | Comma-separated list of schema types used for filtering. Multiple filters can be applied together. |  |
**schema_ids** | Option<**String**> | Comma-separated list of schema ids used for filtering. Multiple filters can be applied together. |  |
**limit** | Option<**i32**> | Specifies the page size for the returned results. Has to be between 10 and 500. Default page size is 50. |  |
**cursor** | Option<**String**> | The cursor that has been returned by the previous result page. Do not pass this parameter if you want to fetch the first page. |  |

### Return type

[**crate::models::SchemaList**](SchemaList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_schema

> crate::models::Schema get_schema(schema_id, fields)
Get a schema

Get a specific schema.    This endpoint returns the content of the latest version of the schema with the content-type `application/json`.    This endpoint requires at least HiveMQ version 4.15.0 on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**schema_id** | **String** | The identifier of the schema. | [required] |
**fields** | Option<**String**> | Comma-separated list of fields to include in the response. Allowed values are: id, type, schemaDefinition, createdAt |  |

### Return type

[**crate::models::Schema**](Schema.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

