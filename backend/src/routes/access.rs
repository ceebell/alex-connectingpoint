use crate::{error::AppError, models::team::Access, state::AppState};
use axum::{extract::FromRequestParts, http::request::Parts};

// Router state is shared; the member context must only live on this request's clone.
pub struct ScopedState(pub AppState);
impl FromRequestParts<AppState> for ScopedState {
    type Rejection = AppError;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let mut scoped = state.clone();
        scoped.access = parts.extensions.get::<Access>().cloned();
        Ok(Self(scoped))
    }
}
