#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bm25Method {
    Robertson,
    Lucene,
    Atire,
    Bm25l,
    Bm25Plus,
}

#[derive(Debug, Clone)]
pub struct Bm25Params {
    pub method: Bm25Method,
    pub k1: f32,
    pub b: f32,
    pub delta: f32,
    pub language: String,
    pub ngrams: u8,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self {
            method: Bm25Method::Bm25Plus,
            k1: 1.2,
            b: 0.75,
            delta: 0.5,
            language: "english".to_string(),
            ngrams: 1,
        }
    }
}

pub fn idf(method: Bm25Method, num_docs: u32, doc_freq: u32) -> f32 {
    let n = num_docs as f64;
    let df = doc_freq as f64;

    let value = match method {
        Bm25Method::Robertson => ((n - df + 0.5) / (df + 0.5)).ln(),
        Bm25Method::Lucene => (1.0 + (n - df + 0.5) / (df + 0.5)).ln(),
        Bm25Method::Atire => (n / df).ln(),
        Bm25Method::Bm25l => ((n + 1.0) / (df + 0.5)).ln(),
        Bm25Method::Bm25Plus => ((n + 1.0) / df).ln(),
    };

    value as f32
}

pub fn tfc(
    method: Bm25Method,
    tf: f32,
    doc_len: f32,
    avg_doc_len: f32,
    k1: f32,
    b: f32,
    delta: f32,
) -> f32 {
    let tf = tf as f64;
    let doc_len = doc_len as f64;
    let avg_doc_len = avg_doc_len as f64;
    let k1 = k1 as f64;
    let b = b as f64;
    let delta = delta as f64;

    let ratio = doc_len / avg_doc_len;
    let b_ratio = b * ratio;
    let norm = 1.0 - b + b_ratio;

    let value = match method {
        Bm25Method::Robertson | Bm25Method::Lucene => tf / (k1 * norm + tf),
        Bm25Method::Atire => {
            let num = tf * (k1 + 1.0);
            let den = tf + k1 * norm;
            num / den
        }
        Bm25Method::Bm25l => {
            let tf_prime = tf + delta;
            let num = (k1 + 1.0) * tf_prime;
            let den = k1 + tf_prime;
            num / den
        }
        Bm25Method::Bm25Plus => {
            let num = (k1 + 1.0) * tf;
            let den = k1 * norm + tf;
            num / den + delta
        }
    };

    value as f32
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn score(
    method: Bm25Method,
    tf: f32,
    doc_len: f32,
    avg_doc_len: f32,
    num_docs: u32,
    doc_freq: u32,
    k1: f32,
    b: f32,
    delta: f32,
) -> f32 {
    let idf_val = idf(method, num_docs, doc_freq) as f64;
    let tfc_val = tfc(method, tf, doc_len, avg_doc_len, k1, b, delta) as f64;
    (idf_val * tfc_val) as f32
}
