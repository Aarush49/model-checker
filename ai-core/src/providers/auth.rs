use anyhow::{Ok, Result};
use keyring::Entry;
use oauth2::{
    AuthType, AuthorizationCode, ClientSecret, EndpointNotSet, EndpointSet, RefreshToken,
    TokenResponse, TokenUrl,
};
use oauth2_reqwest::ReqwestClient;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::RwLock;

use axum::{Router, extract::Query, routing::get};
use oauth2::{
    AuthUrl, ClientId, CsrfToken, PkceCodeChallenge, RedirectUrl, Scope, basic::BasicClient,
};

/// All possible auth states
#[derive(PartialEq, Debug)]
pub(crate) enum AuthStatus {
    /// Refresh token found
    Authenticated,
    /// Refresh token not found
    Unauthenticated,
}

struct TokenState {
    token: String,
    expires_at: OffsetDateTime,
}

/// Each provider can have their differences in oauth, so this tries to handle
/// the differences.
pub(crate) struct OAuthConfig {
    pub provider_name: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_uri: String,
    pub token_uri: String,
    pub scopes: Vec<String>,
    pub extra_auth_params: Vec<(String, String)>,
}

pub(crate) type ConfiguredOAuthClient = oauth2::basic::BasicClient<
    EndpointSet,    // auth_uri is SET
    EndpointNotSet, // device_authorization_url is NOT set
    EndpointNotSet, // introspection_url is NOT set
    EndpointNotSet, // revocation_url is NOT set
    EndpointSet,    // token_uri is SET
>;

pub(crate) struct OAuth {
    oauth_client: ConfiguredOAuthClient,
    oauth_http_client: ReqwestClient,
    http_client: reqwest::Client,
    token: RwLock<TokenState>,
    refresh_entry: Entry,
    config: OAuthConfig,
}

impl OAuth {
    /// Construct new oauth provider with the specified config.
    /// The status of this provider must be checked after constructing it.
    #[must_use]
    pub async fn new(config: OAuthConfig, http_client: &reqwest::Client) -> Result<Self> {
        // Make HTTP client for making oauth related requests
        let oauth_http_client = ReqwestClient::from(http_client.clone());

        // Make the oauth client to handle our oauth related requests
        let mut oauth_client = BasicClient::new(ClientId::new(config.client_id.clone()))
            .set_auth_uri(AuthUrl::new(config.auth_uri.clone())?)
            .set_token_uri(TokenUrl::new(config.token_uri.clone())?)
            .set_auth_type(AuthType::RequestBody);

        // Sometimes the secret will not be specified due to how the provider works.
        // If a provider doesn't require it, we include it in oauth requests
        if let Some(secret) = &config.client_secret {
            oauth_client = oauth_client.set_client_secret(ClientSecret::new(secret.clone()));
        }

        // Make the entry to store the refresh token securely on the user device
        let entry_name = format!("{}-refresh-token", config.provider_name);
        let refresh_entry = Entry::new("modelcheck", &entry_name)?;

        let token = TokenState {
            token: String::new(),
            expires_at: OffsetDateTime::UNIX_EPOCH,
        };

        // Construct the oauth client with temporary token (empty) and temporary expiration
        Ok(Self {
            oauth_client,
            oauth_http_client,
            http_client: http_client.clone(),
            token: RwLock::new(token),
            refresh_entry,
            config: config,
        })
    }

    /// Check if user is authenticated or not by checking if we can find a stored refresh token. If a stored
    /// refresh token is found, attempt to refresh the token. If refreshing succeeds, then the user is
    /// authenthicated.
    #[must_use]
    pub async fn status(&self) -> Result<AuthStatus> {
        Ok(match self.refresh_entry.get_password() {
            std::result::Result::Ok(_) => match self.refresh_token().await {
                std::result::Result::Ok(()) => AuthStatus::Authenticated,
                Err(_) => AuthStatus::Unauthenticated,
            },
            Err(_) => AuthStatus::Unauthenticated,
        })
    }

    /// Login the user using their browser
    #[must_use]
    pub async fn setup(&self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        let auth_client = self
            .oauth_client
            .clone()
            .set_redirect_uri(RedirectUrl::new(format!("http://127.0.0.1:{port}"))?);

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Start building the URL
        let mut auth_request = auth_client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // Dynamically add all scopes from the config
        for scope in &self.config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
        }

        // Dynamically add all extra parameters (like Google's offline access)
        for (key, value) in &self.config.extra_auth_params {
            auth_request = auth_request.add_extra_param(key, value);
        }

        let (auth_url, csrf_token) = auth_request.url();

        let (tx, rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
        let expected_state = csrf_token.secret().clone();

        let oauth_callback = Router::new().route(
            "/",
            get(|Query(params): Query<HashMap<String, String>>| async move {
                let tx = Arc::clone(&tx);
                if let Some(code) = params.get("code")
                    && let Some(state) = params.get("state")
                    && let Some(sender) = tx.lock().unwrap().take()
                {
                    if *state == expected_state {
                        let _ = sender.send(code.clone());
                        "Login successful! You can close this tab."
                    } else {
                        "Security Error: CSRF state mismatch! Login aborted."
                    }
                } else {
                    "Error: No code found"
                }
            }),
        );

        webbrowser::open(auth_url.as_str())?;

        let server_task = tokio::spawn(async move { axum::serve(listener, oauth_callback).await });

        let auth_code = rx.await?;
        server_task.abort();

        let token_result = auth_client
            .exchange_code(AuthorizationCode::new(auth_code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.oauth_http_client)
            .await?;

        if let Some(refresh) = token_result.refresh_token() {
            self.refresh_entry.set_password(refresh.secret().as_str())?;
        }

        let mut token = self.token.write().await;

        token.token = token_result.access_token().secret().to_string();

        let expires = token_result
            .expires_in()
            .unwrap_or(std::time::Duration::from_secs(3600));
        token.expires_at = OffsetDateTime::now_utc() + expires;

        Ok(())
    }

    /// Get the token to use in requests. This value should not be stored.
    #[must_use]
    pub async fn get_token(&self) -> Result<String> {
        let now = OffsetDateTime::now_utc();

        let needs_refresh =
            { self.token.read().await.expires_at <= now + time::Duration::seconds(30) };

        if needs_refresh {
            self.refresh_token().await?;
        }

        Ok(self.token.read().await.token.clone())
    }

    /// Refresh the token with the refresh token.
    async fn refresh_token(&self) -> Result<()> {
        let mut token = self.token.write().await;
        let refresh_secret = self.refresh_entry.get_password()?;

        // Another thread could have already refreshed the token. As a result, we will double check
        let now = OffsetDateTime::now_utc();
        if token.expires_at > now + time::Duration::seconds(30) {
            return Ok(());
        }

        let token_result = self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_secret))
            .request_async(&self.oauth_http_client)
            .await?;

        token.token = token_result.access_token().secret().to_string();

        let expires = token_result
            .expires_in()
            .unwrap_or(Duration::from_secs(3600));
        token.expires_at = OffsetDateTime::now_utc() + expires;

        if let Some(new_refresh) = token_result.refresh_token() {
            self.refresh_entry
                .set_password(new_refresh.secret().as_str())?;
        }

        Ok(())
    }

    pub async fn make_request<T>(&self, request: RequestBuilder) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request = request.bearer_auth(self.get_token().await?).build()?;
        let response = self.http_client.execute(request).await?;

        // 1. Check if the response is successful (e.g., 200 OK)
        if !response.status().is_success() {
            // 2. If it failed, grab the raw text so we can see the exact error message!
            let status = response.status();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error body".to_string());

            // Return a clean error instead of panicking
            return Err(anyhow::anyhow!("API Error {}: {}", status, error_body));
        }

        // 3. If it succeeded, decode it into our struct
        Ok(response.json::<T>().await?)
    }

    /// A tiny helper so providers can start building requests
    pub fn post(&self, url: &str) -> RequestBuilder {
        self.http_client.post(url)
    }
}
