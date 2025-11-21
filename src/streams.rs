use std::fmt;
use std::sync::mpsc::Sender;

type StreamHandler<T> = Box<dyn for<'target> FnOnce(&'target mut T) + Send>;
type ViewHandler<T> =
	Box<dyn for<'target> FnOnce(&'target mut <T as ViewTarget>::View<'target>) + Send>;

/// Message emitted by a background system and delivered to the UI layer.
pub struct StreamEnvelope<M, P> {
	/// Identifier correlating the message with a query or request.
	pub id: u64,
	/// Stream-specific metadata describing the payload.
	pub kind: M,
	/// Payload delivered to the consumer.
	pub payload: P,
	/// Whether the producer finished streaming for this identifier.
	pub complete: bool,
}

impl<M, P> StreamEnvelope<M, P> {
	/// Transform the payload while preserving the envelope metadata.
	pub fn map_payload<N>(self, f: impl FnOnce(P) -> N) -> StreamEnvelope<M, N> {
		StreamEnvelope {
			id: self.id,
			kind: self.kind,
			payload: f(self.payload),
			complete: self.complete,
		}
	}
}

/// Executable payload that knows how to mutate a target value.
pub struct StreamAction<T: ?Sized> {
	handler: StreamHandler<T>,
}

impl<T: ?Sized> StreamAction<T> {
	/// Create a new action from the provided handler.
	pub fn new(handler: impl for<'target> FnOnce(&'target mut T) + Send + 'static) -> Self {
		Self {
			handler: Box::new(handler),
		}
	}

	/// Apply the action to the provided target.
	pub fn apply(self, target: &mut T) {
		(self.handler)(target);
	}
}

impl<M, T: ?Sized> StreamEnvelope<M, StreamAction<T>> {
	/// Execute the action embedded in the envelope against the provided target.
	pub fn dispatch(self, target: &mut T) {
		self.payload.apply(target);
	}
}

/// Sink that can consume [`StreamAction`] payloads.
pub trait EnvelopeSink<T: ?Sized> {
	fn apply(&mut self, action: StreamAction<T>);
}

impl<T: ?Sized> EnvelopeSink<T> for T {
	fn apply(&mut self, action: StreamAction<T>) {
		action.apply(self);
	}
}

impl<T: ?Sized> fmt::Debug for StreamAction<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("StreamAction(..)")
	}
}

/// Marker describing a dynamic view that actions can operate on.
pub trait ViewTarget {
	/// Trait object exposed to the action when executing on the UI thread.
	type View<'target>: ?Sized;
}

/// Executable payload that mutates a dynamic trait object supplied by [`ViewTarget`].
pub struct ViewAction<T: ViewTarget> {
	handler: ViewHandler<T>,
}

impl<T: ViewTarget> ViewAction<T> {
	/// Create a new action from the provided handler.
	pub fn new(
		handler: impl for<'target> FnOnce(&'target mut T::View<'target>) + Send + 'static,
	) -> Self {
		Self {
			handler: Box::new(handler),
		}
	}

	/// Apply the action to the provided view.
	pub fn apply<'view>(self, view: &'view mut T::View<'view>) {
		(self.handler)(view);
	}
}

impl<M, T: ViewTarget> StreamEnvelope<M, ViewAction<T>> {
	/// Execute the embedded action against the provided view.
	pub fn dispatch<'view>(self, view: &'view mut T::View<'view>) {
		self.payload.apply(view);
	}
}

impl<T: ViewTarget> fmt::Debug for ViewAction<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("ViewAction(..)")
	}
}

/// Handle for producing stream messages backed by an [`mpsc::Sender`].
pub struct DataStream<'a, M, P> {
	tx: &'a Sender<StreamEnvelope<M, P>>,
	id: u64,
	kind: M,
}

impl<'a, M: Clone, P: Send + 'static> DataStream<'a, M, P> {
	/// Create a new handle backed by the provided sender.
	#[must_use]
	pub fn new(tx: &'a Sender<StreamEnvelope<M, P>>, id: u64, kind: M) -> Self {
		Self { tx, id, kind }
	}

	/// Identifier associated with this stream.
	#[must_use]
	pub fn id(&self) -> u64 {
		self.id
	}

	/// Metadata associated with each emitted payload.
	#[must_use]
	pub fn kind(&self) -> &M {
		&self.kind
	}

	/// Emit a payload to the consumer.
	pub fn send(&self, payload: P, complete: bool) -> bool {
		self.tx
			.send(StreamEnvelope {
				id: self.id,
				kind: self.kind.clone(),
				payload,
				complete,
			})
			.is_ok()
	}
}

impl<'a, M: Clone, P: Send + 'static> Clone for DataStream<'a, M, P> {
	fn clone(&self) -> Self {
		Self {
			tx: self.tx,
			id: self.id,
			kind: self.kind.clone(),
		}
	}
}
