use crate::{
  http::{
    server_framework::{Endpoint, EndpointNode, Middleware, RouteMatch},
    AutoStream, ManualStream, OperationMode, Response, StatusCode,
  },
  misc::{ArrayVector, Vector},
};
use core::{marker::PhantomData, ops::ControlFlow};

/// Redirects requests to specific asynchronous functions based on the set of inner URIs.
#[derive(Debug)]
pub struct Router<CA, E, EN, M, S, SA> {
  pub(crate) en: EN,
  #[cfg(feature = "matchit")]
  pub(crate) _matcher: matchit::Router<(ArrayVector<RouteMatch, 4>, OperationMode)>,
  #[cfg(not(feature = "matchit"))]
  pub(crate) _matcher:
    (hashbrown::HashMap<alloc::string::String, OperationMode>, Option<OperationMode>),
  pub(crate) middlewares: M,
  pub(crate) phantom: PhantomData<(CA, E, S, SA)>,
}

impl<CA, E, EN, M, S, SA> Router<CA, E, EN, M, S, SA>
where
  E: From<crate::Error>,
  EN: EndpointNode<CA, E, S, SA>,
{
  /// Creates a new instance with generic paths and middlewares.
  #[inline]
  pub fn new(en: EN, middlewares: M) -> crate::Result<Self> {
    let _matcher = Self::_matcher(&en)?;
    Ok(Self { _matcher, middlewares, en, phantom: PhantomData })
  }

  #[inline]
  #[cfg(feature = "matchit")]
  fn _matcher(
    en: &EN,
  ) -> crate::Result<matchit::Router<(ArrayVector<RouteMatch, 4>, OperationMode)>> {
    let mut vec = Vector::new();
    en.paths_indices(ArrayVector::new(), &mut vec)?;
    let mut _matcher = matchit::Router::new();
    for array in vec {
      let [initials @ .., last] = array.as_slice() else {
        continue;
      };
      let mut key = alloc::string::String::new();
      for elem in initials {
        key.push_str(elem.path);
      }
      key.push_str(last.path);
      let om = last.om;
      _matcher.insert(key, (array, om))?;
    }
    Ok(_matcher)
  }

  #[inline]
  #[cfg(not(feature = "matchit"))]
  fn _matcher(
    paths: &EN,
  ) -> crate::Result<(
    hashbrown::HashMap<alloc::string::String, OperationMode>,
    Option<OperationMode>,
  )> {
    let mut paths_indices = Vector::new();
    paths.paths_indices(ArrayVector::new(), &mut paths_indices)?;
    let mut paths = hashbrown::HashMap::new();
    if let Some(om) = Self::unique_om(&paths_indices) {
      return Ok((paths, Some(om)));
    }
    for array in paths_indices {
      let [first, ..] = array.as_slice() else {
        continue;
      };
      let _ = paths.insert(first.path.into(), first.om);
    }
    Ok((paths, None))
  }

  #[inline]
  #[cfg(not(feature = "matchit"))]
  fn unique_om(paths_indices: &[ArrayVector<RouteMatch, 4>]) -> Option<OperationMode> {
    let mut array_iter = paths_indices.iter();
    let Some(first_array) = array_iter.next() else {
      return None;
    };
    let mut first_array_iter = first_array.iter();
    let Some(first_elem) = first_array_iter.next() else {
      return None;
    };
    let om = first_elem.om;
    for elem in first_array_iter {
      if elem.om != om {
        return None;
      }
    }
    for array in array_iter {
      for elem in array {
        if elem.om != om {
          return None;
        }
      }
    }
    Some(om)
  }
}

impl<CA, E, EN, S, SA> Router<CA, E, EN, (), S, SA>
where
  E: From<crate::Error>,
  EN: EndpointNode<CA, E, S, SA>,
{
  /// Creates a new instance with automatic paths and middlewares.
  #[inline]
  pub fn paths(en: EN) -> crate::Result<Self> {
    let _matcher = Self::_matcher(&en)?;
    Ok(Self { en, _matcher, middlewares: (), phantom: PhantomData })
  }
}

impl<CA, E, EN, M, S, SA> Endpoint<CA, E, S, SA> for Router<CA, E, EN, M, S, SA>
where
  E: From<crate::Error>,
  EN: EndpointNode<CA, E, S, SA>,
  M: Middleware<CA, E, SA>,
{
  const OM: OperationMode = OperationMode::Auto;

  #[inline]
  async fn auto(
    &self,
    auto_stream: &mut AutoStream<CA, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<StatusCode, E> {
    let mw_aux = &mut self.middlewares.aux();
    if let ControlFlow::Break(el) = self
      .middlewares
      .req(&mut auto_stream.conn_aux, mw_aux, &mut auto_stream.req, &mut auto_stream.stream_aux)
      .await?
    {
      return Ok(el);
    }
    let status_code = self.en.auto(auto_stream, path_defs).await?;
    if let ControlFlow::Break(el) = self
      .middlewares
      .res(
        &mut auto_stream.conn_aux,
        mw_aux,
        Response { rrd: &mut auto_stream.req.rrd, status_code, version: auto_stream.req.version },
        &mut auto_stream.stream_aux,
      )
      .await?
    {
      return Ok(el);
    }
    Ok(status_code)
  }

  #[inline]
  async fn manual(
    &self,
    mut manual_stream: ManualStream<CA, S, SA>,
    path_defs: (u8, &[RouteMatch]),
  ) -> Result<(), E> {
    let mw_aux = &mut self.middlewares.aux();
    if let ControlFlow::Break(_) = self
      .middlewares
      .req(
        &mut manual_stream.conn_aux,
        mw_aux,
        &mut manual_stream.req,
        &mut manual_stream.stream_aux,
      )
      .await?
    {
      return Ok(());
    }
    self.en.manual(manual_stream, path_defs).await?;
    Ok(())
  }
}

impl<CA, E, EN, M, S, SA> EndpointNode<CA, E, S, SA> for Router<CA, E, EN, M, S, SA>
where
  E: From<crate::Error>,
  EN: EndpointNode<CA, E, S, SA>,
  M: Middleware<CA, E, SA>,
{
  const IS_ROUTER: bool = true;

  #[inline]
  fn paths_indices(
    &self,
    prev: ArrayVector<RouteMatch, 4>,
    vec: &mut Vector<ArrayVector<RouteMatch, 4>>,
  ) -> crate::Result<()> {
    self.en.paths_indices(prev, vec)
  }
}
