use super::super::{Authentication, CustomError};
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use db::authz;
use db::queries::conversations;
use db::{Conversation, Pool};

pub async fn index(
    Path(team_id): Path<i32>,
    current_user: Authentication,
    Extension(pool): Extension<Pool>,
) -> Result<impl IntoResponse, CustomError> {
    let mut client = pool.get().await?;
    let transaction = client.transaction().await?;

    let _permissions = authz::get_permissions(&transaction, &current_user.into(), team_id).await?;

    // Get the latest conversation
    let conversation: Result<Conversation, db::TokioPostgresError> =
        conversations::get_latest_conversation()
            .bind(&transaction)
            .one()
            .await;

    let conv_id = if let Ok(conversation) = conversation {
        conversation.id
    } else {
        conversations::create_conversation()
            .bind(&transaction, &team_id)
            .one()
            .await?
    };

    transaction.commit().await?;

    super::super::layout::redirect(&ui_pages::routes::console::conversation_route(
        team_id, conv_id,
    ))
}
