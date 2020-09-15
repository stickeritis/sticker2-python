use std::fs::File;

use anyhow::{Context, Result};
use sticker2::config::{Config, PretrainConfig};
use sticker2::encoders::Encoders;
use sticker2::input::Tokenize;
use sticker2::model::bert::BertModel;
use tch::nn::VarStore;
use tch::Device;

/// Wrapper around different parts of a model.
pub struct Model {
    pub encoders: Encoders,
    pub model: BertModel,
    pub tokenizer: Box<dyn Tokenize>,
    pub vs: VarStore,
}

impl Model {
    /// Load a model on the given device.
    pub fn load(config: &Config, device: Device) -> Result<Model> {
        let encoders = load_encoders(&config)?;
        let tokenizer = load_tokenizer(&config)?;
        let pretrain_config = load_pretrain_config(&config)?;

        let mut vs = VarStore::new(device);

        let model = BertModel::new(
            vs.root(),
            &pretrain_config,
            &encoders,
            0.0,
            config.model.position_embeddings.clone(),
        )
        .context("Cannot construct model")?;

        vs.load(&config.model.parameters)
            .context("Cannot load model parameters")?;

        vs.freeze();

        Ok(Model {
            encoders,
            model,
            tokenizer,
            vs,
        })
    }
}

pub fn load_pretrain_config(config: &Config) -> Result<PretrainConfig> {
    config
        .model
        .pretrain_config()
        .context("Cannot load pretraining model configuration")
}

fn load_encoders(config: &Config) -> Result<Encoders> {
    let f = File::open(&config.labeler.labels)
        .context(format!("Cannot open label file: {}", config.labeler.labels))?;
    let encoders: Encoders = serde_yaml::from_reader(&f).context(format!(
        "Cannot deserialize labels from: {}",
        config.labeler.labels
    ))?;

    for encoder in &*encoders {
        eprintln!(
            "Loaded labels for encoder '{}': {} labels",
            encoder.name(),
            encoder.encoder().len()
        );
    }

    Ok(encoders)
}

pub fn load_tokenizer(config: &Config) -> Result<Box<dyn Tokenize>> {
    config
        .tokenizer()
        .context("Cannot read tokenizer vocabulary")
}
