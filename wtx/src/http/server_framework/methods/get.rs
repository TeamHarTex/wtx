use crate::{
  http::{
    server_framework::{methods::check_method, Endpoint, EndpointNode, RouteMatch},
    AutoStream, ManualStream, Method, OperationMode, StatusCode,
  },
  misc::{ArrayVector, FnFut, Vector},
};

/// Requires a request of type `GET`.
#[derive(Debug)]
pub struct Get<T>(
  /// Arbitrary type
  pub T,
);

/// Creates a new [`Get`] instance.
#[inline]
pub fn get<A, T>(ty: T) -> Get<T::Wrapper>
where
  T: FnFut<A>,
{
  Get(ty.into_wrapper())
}

impl<CA, E, S, SA, T> Endpoint<CA, E, S, SA> for Get<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
{
  const OM: OperationMode = T::OM;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    check_method(Method::Get, auto_stream.req.method)?;
    self.0.auto(auto_stream, path_defs).await
  }

  #[inline]
  async fn manual(
    &self,
    manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    check_method(Method::Get, manual_stream.req.method)?;
    self.0.manual(manual_stream, path_defs).await
  }
}

impl<CA, E, S, SA, T> EndpointNode<CA, E, S, SA> for Get<T>
where
  E: From<crate::Error>,
  T: Endpoint<CA, E, S, SA>,
{
  const IS_ROUTER: bool = false;

  #[inline]
  fn paths_indices(
    &self,
    _: ArrayVector<RouteMatch, 4>,
    _: &mut Vector<ArrayVector<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    Ok(())
  }
}
