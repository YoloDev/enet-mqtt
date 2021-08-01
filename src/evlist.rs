use async_stream::stream;
use eventuals::{EventualReader, Value};
use futures::{stream::SelectAll, Stream, StreamExt};
use std::{
  pin::Pin,
  task::{Context, Poll},
};

pub struct EvList<E, V> {
  stream: SelectAll<ValueStream<E, V>>,
}

impl<E, V> Stream for EvList<E, V> {
  type Item = ValueStreamResult<E, V>;

  fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.stream.poll_next_unpin(cx)
  }
}

impl<E, V> EvList<E, V>
where
  E: Clone + 'static,
  V: Value,
{
  pub fn new(entities: Vec<(E, EventualReader<V>)>) -> Self {
    let stream = entities
      .into_iter()
      .map(|(e, v)| ValueStream::new(e, v))
      .collect();

    Self { stream }
  }
}

type ValueStreamResult<E, V> = (E, V);
type ValueStreamDyn<E, V> = dyn Stream<Item = ValueStreamResult<E, V>>;

struct ValueStream<E, V>(Pin<Box<ValueStreamDyn<E, V>>>);

impl<E, V> ValueStream<E, V>
where
  E: Clone + 'static,
  V: Value,
{
  fn new(entity: E, mut reader: EventualReader<V>) -> Self {
    let stream = stream! {
      while let Ok(value) = reader.next().await {
        yield (entity.clone(), value);
      }
    };

    Self(Box::pin(stream))
  }
}

impl<E, V> Stream for ValueStream<E, V> {
  type Item = ValueStreamResult<E, V>;

  fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    self.get_mut().0.poll_next_unpin(cx)
  }
}
