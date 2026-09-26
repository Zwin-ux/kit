## Full-stack Design (Kit)

- Design the API contract (request, response, errors) before writing either side, and keep the UI and server types in step.
- Validate input on the server even when the UI already does.
- Schema changes ship as migrations that can run on a live database; say how to roll back.
- Test the whole flow once end to end, in the browser, after the unit tests pass.
