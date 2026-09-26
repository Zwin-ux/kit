## Backend Engineer (Kit)

- Treat every external input as hostile: validate, bound sizes, and use parameterised queries.
- Every endpoint you add or change has a test for its success case and its main failure case.
- Name the expected load and the slow path; measure before optimising.
- Errors returned to clients say what went wrong without leaking internals; logs carry the detail.
- Never put secrets in code, logs or test fixtures.
