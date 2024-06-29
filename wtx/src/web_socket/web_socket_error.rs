/// WebSocket Error
#[derive(Debug)]
pub enum WebSocketError {
  /// It it not possible to read a frame of a connection that was previously closed.
  ConnectionClosed,
  /// HTTP headers must be unique.
  DuplicatedHeader,
  /// The requested received in a handshake on a server is not valid.
  InvalidAcceptRequest,
  /// Received close frame has invalid parameters.
  InvalidCloseFrame,
  /// Received an invalid header compression parameter.
  InvalidCompressionHeaderParameter,
  /// Header indices are out-of-bounds or the number of bytes are too small.
  InvalidFrameHeaderBounds,
  /// Payload indices are out-of-bounds or the number of bytes are too small.
  InvalidPayloadBounds,
  /// Server received a frame without a mask.
  MissingFrameMask,
  /// Client sent "permessage-deflate" but didn't receive back from the server
  MissingPermessageDeflate,
  /// Status code is expected to be
  MissingSwitchingProtocols,
  /// Server responded without a compression context but the client does not allow such behavior.
  NoCompressionContext,
  /// Reserved bits are not zero.
  ReservedBitsAreNotZero,
  /// Received control frame wasn't supposed to be fragmented.
  UnexpectedFragmentedControlFrame,
  /// The first frame of a message is a continuation or the following frames are not a
  /// continuation.
  UnexpectedMessageFrame,
  /// Control frames have a maximum allowed size.
  VeryLargeControlFrame,
  /// Frame payload exceeds the defined threshold.
  VeryLargePayload,
}
