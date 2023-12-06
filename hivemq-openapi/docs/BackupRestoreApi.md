# \BackupRestoreApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_backup**](BackupRestoreApi.md#create_backup) | **POST** /api/v1/management/backups | Create a new backup
[**download_backup_file**](BackupRestoreApi.md#download_backup_file) | **GET** /api/v1/management/files/backups/{backupId} | Download a backup file
[**get_all_backups**](BackupRestoreApi.md#get_all_backups) | **GET** /api/v1/management/backups | List all available backups
[**get_backup**](BackupRestoreApi.md#get_backup) | **GET** /api/v1/management/backups/{backupId} | Get backup information
[**restore_backup**](BackupRestoreApi.md#restore_backup) | **POST** /api/v1/management/backups/{backupId} | Restore a new backup



## create_backup

> crate::models::BackupItem create_backup()
Create a new backup

Triggers the creation of a new backup.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::BackupItem**](BackupItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## download_backup_file

> std::path::PathBuf download_backup_file(backup_id)
Download a backup file

Download a specific backup file.    This endpoint returns the content of the backup file with the content-type `application/octet-stream`.    Only backups in the states `COMPLETED`, `RESTORE_IN_PROGRESS`, `RESTORE_FAILED` or `RESTORE_COMPLETED` can be downloaded.   This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **String** | The id of the backup. | [required] |

### Return type

[**std::path::PathBuf**](std::path::PathBuf.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/octet-stream, application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_all_backups

> crate::models::BackupList get_all_backups()
List all available backups

Lists all available backups with their current state.  This endpoint can be used to get an overview over all backups that are in progress or can be restored.  Canceled or failed backups are included in the results for up to 1 hour after they have been requested.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.

### Parameters

This endpoint does not need any parameter.

### Return type

[**crate::models::BackupList**](BackupList.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_backup

> crate::models::BackupItem get_backup(backup_id)
Get backup information

Returns the information for a specific backup with its current state.   This endpoint can be used to check the progress of a specific backup when it is being created or being restored.    Canceled or failed backups are returned for up to 1 hour after the have been requested.   This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **String** | The id of the backup. | [required] |

### Return type

[**crate::models::BackupItem**](BackupItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## restore_backup

> crate::models::BackupItem restore_backup(backup_id)
Restore a new backup

Triggers the restore of a stored backup.  This endpoint requires at least HiveMQ version 4.4.0. on all cluster nodes.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**backup_id** | **String** | The id of the backup. | [required] |

### Return type

[**crate::models::BackupItem**](BackupItem.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

