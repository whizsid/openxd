# Opening a local file

The editor should allow to open a local OpenXD file.

Once user selected a compressed local file, it must be uploaded, extracted
and cached in a caching service and backend should informed. Also backend 
should has the access to this cache.

## Desktop Version

Desktop version can use the user's file system for caching. `cache_dir`
from `dirs` crate can be used to store cache. Since there should be large
files, we have to avoid using the channels to send those binaries to backend.
We can store those files directly in the UI thread and inform the backend.
Backend will be informed with a path like below.

```
"${CACHE_DIR}/OpenXD/local/%2Fhome%2Falex%2FProjects%2Fmy-new-website.oxd"
```

If user accidentally closed the application without saving the file, then
this path will remain and user will be informed in next time about unsaved
file.

## Web Version

Since we can not store large files in user's space in browsers, we have to
store those cache files in server side. So we have to upload the file once user
selected a file. Also we can not use the existing web socket connection to upload
files due to large size. So we have to provide an external API.
