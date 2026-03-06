# Redmine API Reference — rrmcp

Target: Redmine **stable** release. All resources listed in the official Redmine REST API docs.

Auth: `X-Redmine-API-Key: <user-api-key>` header on every request.
Base: `{REDMINE_BASE_URL}/`

## Resource Coverage

| Resource            | Status    | Since | Notes                                                      |
| ------------------- | --------- | ----- | ---------------------------------------------------------- |
| Issues              | Stable    | 1.0   |                                                            |
| Projects            | Stable    | 1.0   |                                                            |
| Users               | Stable    | 1.1   | Requires admin for listing all users                       |
| Time Entries        | Stable    | 1.1   |                                                            |
| News                | Prototype | 1.1   | Index only (list), no create/update via API                |
| Project Memberships | Alpha     | 1.4   |                                                            |
| Issue Relations     | Alpha     | 1.3   |                                                            |
| Versions            | Alpha     | 1.3   |                                                            |
| Queries             | Alpha     | 1.3   |                                                            |
| Issue Statuses      | Alpha     | 1.3   | Read-only list                                             |
| Trackers            | Alpha     | 1.3   | Read-only list                                             |
| Issue Categories    | Alpha     | 1.3   |                                                            |
| Roles               | Alpha     | 1.4   |                                                            |
| Attachments         | Beta      | 1.3   | Upload via multipart (added in 1.4)                        |
| Groups              | Alpha     | 2.1   |                                                            |
| Wiki Pages          | Alpha     | 2.2   |                                                            |
| Enumerations        | Alpha     | 2.2   | Issue priorities, time tracking activities, doc categories |
| Custom Fields       | Alpha     | 2.4   | Read-only list                                             |
| Search              | Alpha     | 3.3   |                                                            |
| Files               | Alpha     | 3.4   |                                                            |
| My Account          | Alpha     | 4.1   |                                                            |
| Journals            | Alpha     | 5.0   | Issue change history                                       |

## Key API Patterns

### Pagination

Most list endpoints support:

- `offset` — number of records to skip
- `limit` — number of records to return (max 100 by default)
- Response includes `total_count`, `offset`, `limit`

### Formats

- All requests/responses use JSON (`Content-Type: application/json`)
- URL pattern: `/resource.json` or `/resource/{id}.json`

### Includes

Many resources support `?include=` param to embed related data:

- Issues: `journals,relations,attachments,changesets,watchers,allowed_statuses`
- Projects: `trackers,issue_categories,enabled_modules,time_entry_activities,issue_custom_fields`

## Issues API

```
GET    /issues.json                    List issues (filterable)
POST   /issues.json                    Create issue
GET    /issues/{id}.json               Get issue
PUT    /issues/{id}.json               Update issue
DELETE /issues/{id}.json               Delete issue
```

Key filters: `project_id`, `tracker_id`, `status_id`, `assigned_to_id`, `subject`, `created_on`, `updated_on`

## Projects API

```
GET    /projects.json                  List projects
POST   /projects.json                  Create project
GET    /projects/{id}.json             Get project (id or identifier)
PUT    /projects/{id}.json             Update project
DELETE /projects/{id}.json             Delete project (irreversible)
GET    /projects/{id}/memberships.json List memberships
POST   /projects/{id}/memberships.json Add membership
GET    /projects/{id}/versions.json    List versions
POST   /projects/{id}/versions.json    Create version
GET    /projects/{id}/issue_categories.json
POST   /projects/{id}/issue_categories.json
GET    /projects/{id}/wiki/index.json  List wiki pages
```

## Wiki Pages API

```
GET    /projects/{id}/wiki/index.json  List all pages
GET    /projects/{id}/wiki/{title}.json  Get page
PUT    /projects/{id}/wiki/{title}.json  Create or update page
DELETE /projects/{id}/wiki/{title}.json  Delete page
```

## Time Entries API

```
GET    /time_entries.json              List (filter by project, issue, user, date)
POST   /time_entries.json              Create
GET    /time_entries/{id}.json         Get
PUT    /time_entries/{id}.json         Update
DELETE /time_entries/{id}.json         Delete
```

## Attachments API

```
POST   /uploads.json                   Upload file (binary), returns token
PUT    /issues/{id}.json               Attach by adding token to uploads[] in body
GET    /attachments/{id}.json          Get attachment metadata
DELETE /attachments/{id}.json          Delete attachment
```

## Search API

```
GET    /search.json?q={query}&...      Search across all resources
```

Optional filters: `scope`, `all_words`, `titles_only`, `issues`, `news`, `documents`, `changesets`, `wiki_pages`, `messages`, `projects`

## Users API

```
GET    /users.json                     List users (admin only)
POST   /users.json                     Create user (admin only)
GET    /users/{id}.json                Get user
PUT    /users/{id}.json                Update user (admin only)
DELETE /users/{id}.json                Delete user (admin only)
GET    /users/current.json             Get current user
```

## My Account API

```
GET    /my/account.json                Get current user account
PUT    /my/account.json                Update current user account
```

## Journals API (Issue History)

Journals are included via `?include=journals` on issue requests.
No standalone endpoint — always fetched as part of an issue.

## Enumerations API

```
GET    /enumerations/issue_priorities.json
GET    /enumerations/time_entry_activities.json
GET    /enumerations/document_categories.json
```

## Custom Fields API

```
GET    /custom_fields.json             List all custom fields (admin only)
```

## Known Quirks / Gotchas

- Redmine returns `422 Unprocessable Entity` with `{"errors": [...]}` on validation failures
- Some resources return `200` even on partial failures — always check the response body
- Deleting a project is **irreversible** and cascades to all sub-resources
- API key must have appropriate Redmine role permissions — not all API calls are available to all users
- `News` resource only supports listing (index), not create/update/delete via API
- Journals cannot be created or deleted via API — they are auto-generated by Redmine on issue changes
- File upload (attachments) is a two-step process: upload first, then attach to issue
