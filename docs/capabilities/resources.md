# Packaged resources

`framework-resources` provides read-only lookup for resources packaged with an application. Its
public API types are `ResourcePath<'a>`, `ResourceBackend`, `ResourceError`, and `Resources<B>`.
Give it a borrowed UTF-8 slash-separated relative path, such as `images/logo.png`, and a statically
selected `ResourceBackend`. The facade has no global registry or initialization. `read` returns
caller-owned bytes; `read_string` converts those bytes into an owned UTF-8 `String` by consuming
the returned vector, without a second facade-level byte copy.

## API contract

`ResourcePath::new(relative)` validates but preserves the exact supplied path, and
`ResourcePath::relative()` returns that same borrowed text. A backend implements
`ResourceBackend::availability` and `ResourceBackend::read`; the latter receives the borrowed
`ResourcePath` and returns an owned `Vec<u8>` or `ResourceError`. Wrap the backend with
`Resources::new(backend)`, then call `availability`, `read`, or `read_string` on the facade.
`Resources::into_backend` returns the same backend value.

`ResourceError::InvalidPath` and `ResourceError::InvalidUtf8` map to
`ErrorKind::InvalidInput`. `ResourceError::Backend(error)` preserves `error.kind()` and
`error.platform_code()`; `ResourceBackend::read` must use `ErrorKind::NotFound` for an absent path.
The Rust signatures are:

```text
ResourcePath::new(relative: &'a str) -> Result<ResourcePath<'a>, ResourceError>
ResourcePath::relative(self) -> &'a str
ResourceBackend::availability(&self) -> Availability
ResourceBackend::read(&mut self, path: ResourcePath<'_>) -> Result<Vec<u8>, ResourceError>
Resources::new(backend: B) -> Resources<B>
Resources::availability(&self) -> Availability
Resources::read(&mut self, path: ResourcePath<'_>) -> Result<Vec<u8>, ResourceError>
Resources::read_string(&mut self, path: ResourcePath<'_>) -> Result<String, ResourceError>
Resources::into_backend(self) -> B
```

`ResourcePath::new` rejects an empty or absolute path, empty path components, `.` or `..`
components, backslashes, and NUL bytes. It preserves the supplied text: it does not normalize
names, and no Unicode normalization or case-folding behavior is promised. A backend reports a
missing resource with `ErrorKind::NotFound`; `ResourceError` preserves the backend's `ErrorKind`
and optional platform error code.

Lookup is synchronous and may block. The backend defines platform availability, bundle location,
file-system security, and resource-copy costs. The contract does not define cancellation, directory
enumeration, arbitrary URL lookup, localized-resource selection, asset-catalog access, or resource
format decoding. Packaged resources are read-only and distinct from writable sandbox files.
