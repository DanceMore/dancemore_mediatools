# Jules Rules SDLC

## Stack Mandates
- **Backend**: Rust using the Rocket framework.
- **Frontend**: HTML templates using Minijinja (via `rocket_dyn_templates`).
- **Workspace**: Cargo-based multi-project repository.

## Verification Mandates
- **Every PR must include a test.**
- **Web Verification**: For changes in `tv_mode_web`, use Rust integration tests (`rocket::local`) to verify template rendering and route integrity.
- **Test Automation**: All tests must be runnable via `make test`.

## Documentation & Specs
- **Specification First**: Always look for and refer to `.md` files in `docs/specs/` before implementing features or changes.
- **Architecture**: Maintain a clean separation between route handlers, application state, and external service proxies.
