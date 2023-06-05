use crate::layers::Layer;
use metrics::{Counter, Gauge, Histogram, Key, KeyName, Recorder, SharedString, Attribute};

/// Applies a prefix to every metric key.
///
/// Keys will be prefixed in the format of `<prefix>.<remaining>`.
pub struct Prefix<R> {
    prefix: SharedString,
    inner: R,
}

impl<R> Prefix<R> {
    fn prefix_key(&self, key: &Key) -> Key {
        let mut new_name = String::with_capacity(self.prefix.len() + 1 + key.name().len());
        new_name.push_str(self.prefix.as_ref());
        new_name.push('.');
        new_name.push_str(key.name());

        Key::from_parts(new_name, key.labels())
    }

    fn prefix_key_name(&self, key_name: KeyName) -> KeyName {
        let mut new_name = String::with_capacity(self.prefix.len() + 1 + key_name.as_str().len());
        new_name.push_str(self.prefix.as_ref());
        new_name.push('.');
        new_name.push_str(key_name.as_str());

        KeyName::from(new_name)
    }
}

impl<R: Recorder> Recorder for Prefix<R> {
    fn set_counter_attribute(&self, key: KeyName, attribute: Box<dyn Attribute>) {
        let new_key = self.prefix_key_name(key);
        self.inner.set_counter_attribute(new_key, attribute)
    }

    fn set_gauge_attribute(&self, key: KeyName, attribute: Box<dyn Attribute>) {
        let new_key = self.prefix_key_name(key);
        self.inner.set_gauge_attribute(new_key, attribute)
    }

    fn set_histogram_attribute(&self, key: KeyName, attribute: Box<dyn Attribute>) {
        let new_key = self.prefix_key_name(key);
        self.inner.set_histogram_attribute(new_key, attribute)
    }

    fn register_counter(&self, key: &Key) -> Counter {
        let new_key = self.prefix_key(key);
        self.inner.register_counter(&new_key)
    }

    fn register_gauge(&self, key: &Key) -> Gauge {
        let new_key = self.prefix_key(key);
        self.inner.register_gauge(&new_key)
    }

    fn register_histogram(&self, key: &Key) -> Histogram {
        let new_key = self.prefix_key(key);
        self.inner.register_histogram(&new_key)
    }
}

/// A layer for applying a prefix to every metric key.
///
/// More information on the behavior of the layer can be found in [`Prefix`].
pub struct PrefixLayer(&'static str);

impl PrefixLayer {
    /// Creates a new `PrefixLayer` based on the given prefix.
    pub fn new<S: Into<String>>(prefix: S) -> PrefixLayer {
        PrefixLayer(Box::leak(prefix.into().into_boxed_str()))
    }
}

impl<R> Layer<R> for PrefixLayer {
    type Output = Prefix<R>;

    fn layer(&self, inner: R) -> Self::Output {
        Prefix { prefix: self.0.into(), inner }
    }
}

#[cfg(test)]
mod tests {
    use super::{Prefix, PrefixLayer};
    use crate::layers::Layer;
    use crate::test_util::*;
    use metrics::{Counter, Gauge, Histogram, Key, KeyName, attributes::Description};

    #[test]
    fn test_basic_functionality() {
        let inputs = vec![
            RecorderOperation::SetCounterAttribute(
                "counter_key".into(),
                Box::new(Description::from("counter desc")),
            ),
            RecorderOperation::SetGaugeAttribute(
                "gauge_key".into(),
                Box::new(Description::from("gauge desc")),
            ),
            RecorderOperation::SetHistogramAttribute(
                "histogram_key".into(),
                Box::new(Description::from("histogram desc")),
            ),
            RecorderOperation::RegisterCounter("counter_key".into(), Counter::noop()),
            RecorderOperation::RegisterGauge("gauge_key".into(), Gauge::noop()),
            RecorderOperation::RegisterHistogram("histogram_key".into(), Histogram::noop()),
        ];

        let expectations = vec![
            RecorderOperation::SetCounterAttribute(
                "testing.counter_key".into(),
                Box::new(Description::from("counter desc")),
            ),
            RecorderOperation::SetGaugeAttribute(
                "testing.gauge_key".into(),
                Box::new(Description::from("gauge desc")),
            ),
            RecorderOperation::SetHistogramAttribute(
                "testing.histogram_key".into(),
                Box::new(Description::from("histogram desc")),
            ),
            RecorderOperation::RegisterCounter("testing.counter_key".into(), Counter::noop()),
            RecorderOperation::RegisterGauge("testing.gauge_key".into(), Gauge::noop()),
            RecorderOperation::RegisterHistogram("testing.histogram_key".into(), Histogram::noop()),
        ];

        let recorder = MockBasicRecorder::from_operations(expectations);
        let prefix = PrefixLayer::new("testing");
        let prefix = prefix.layer(recorder);

        for operation in inputs {
            operation.apply_to_recorder(&prefix);
        }
    }

    #[test]
    fn test_key_vs_key_name() {
        let prefix = Prefix { prefix: "foobar".into(), inner: () };

        let key_name = KeyName::from("my_key");
        let key = Key::from_name(key_name.clone());

        let prefixed_key = prefix.prefix_key(&key);
        let prefixed_key_name = prefix.prefix_key_name(key_name);

        assert_eq!(
            prefixed_key.name(),
            prefixed_key_name.as_str(),
            "prefixed key and prefixed key name should match"
        );
    }
}
