use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use std::ops::Deref;

macro_rules! boolean_query_param {
    ($i:ident, $name:expr) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $i(pub bool);

        #[async_trait]
        impl<S> FromRequestParts<S> for $i
        where
            S: Send + Sync,
        {
            type Rejection = String;

            async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
                parts
                    .uri
                    .query()
                    .and_then(|query| {
                        form_urlencoded::parse(query.as_bytes()).find(|(k, _)| k == $name)
                    })
                    .map(|(_, value)| match &*value {
                        "" => Ok(true),
                        _ => value.parse::<bool>(),
                    })
                    .unwrap_or(Ok(false))
                    .map($i)
                    .map_err(|err| format!("Query parameter `{}` invalid: {err}", $name))
            }
        }

        impl Deref for $i {
            type Target = bool;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

boolean_query_param!(Login, "login");
boolean_query_param!(Unlock, "unlock");
boolean_query_param!(CreateRestore, "restore");
boolean_query_param!(ShareRestore, "share");
boolean_query_param!(Progress, "progress");

#[cfg(test)]
mod does {
    use super::*;
    use axum::{extract::FromRequestParts, http::Request};

    async fn eval_query<T: FromRequestParts<()> + Deref<Target = bool>>(
        query: &str,
    ) -> Result<bool, T::Rejection> {
        let request = Request::get(format!("http://localhost?{query}"))
            .body(())
            .unwrap();

        let (mut parts, _) = request.into_parts();

        T::from_request_parts(&mut parts, &())
            .await
            .map(|entry| *entry)
    }

    macro_rules! query {
        ($t:ty, [ $( $query:expr => $result:expr ),* ]) => {
            $(
                assert_eq!(eval_query::<$t>($query).await, $result, "query was `{}`", $query);
            )*
        };
    }

    #[tokio::test]
    async fn parse_boolean_as_expected() {
        query!(Login, [
            "link=true" => Ok(true),
            "link" => Ok(true),

            "link=false" => Ok(false),
            "" => Ok(false),

            "link=1" => Err("Query parameter `link` invalid: provided string was not `true` or `false`".into())
        ]);
    }
}
