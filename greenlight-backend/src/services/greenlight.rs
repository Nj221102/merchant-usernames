use crate::config::Config;
use crate::error::{AppError, Result};
use crate::handlers::node::CreateOfferRequest;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

use gl_client::{
    bitcoin::Network,
    credentials::{Device, Nobody},
    scheduler::Scheduler,
    signer::Signer,
    node::ClnClient,
    pb::cln::OfferRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCredentials {
    pub creds: Vec<u8>,
}

pub struct GreenlightService {
    #[allow(dead_code)]
    config: Config,
}

impl GreenlightService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn register_node(&self, seed: &[u8]) -> Result<DeviceCredentials> {
        // Load developer credentials from files
        let developer_cert = fs::read(&self.config.gl_cert_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read cert file: {}", e)))?;
        
        let developer_key = fs::read(&self.config.gl_key_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read key file: {}", e)))?;
        
        // Create Nobody credentials with developer cert/key
        let developer_creds = Nobody {
            cert: developer_cert,
            key: developer_key,
            ..Nobody::default()
        };

        // Create scheduler with developer credentials
        let scheduler = Scheduler::new(Network::Bitcoin, developer_creds.clone())
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to create scheduler: {}", e)))?;

        // Create a signer from the seed
        let signer = Signer::new(seed.to_vec(), Network::Bitcoin, developer_creds)
            .map_err(|e| AppError::Greenlight(format!("Failed to create signer: {}", e)))?;

        // Register the node
        let registration_response = scheduler
            .register(&signer, None)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to register node: {}", e)))?;

        // Return device credentials as bytes (they're already in the registration response)
        Ok(DeviceCredentials {
            creds: registration_response.creds,
        })
    }

    pub async fn recover_node(&self, seed: &[u8]) -> Result<DeviceCredentials> {
        // Load developer credentials from files
        let developer_cert = fs::read(&self.config.gl_cert_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read cert file: {}", e)))?;
        
        let developer_key = fs::read(&self.config.gl_key_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read key file: {}", e)))?;
        
        // Create Nobody credentials with developer cert/key
        let developer_creds = Nobody {
            cert: developer_cert,
            key: developer_key,
            ..Nobody::default()
        };

        // Create scheduler with developer credentials
        let scheduler = Scheduler::new(Network::Bitcoin, developer_creds.clone())
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to create scheduler: {}", e)))?;

        // Create a signer from the seed
        let signer = Signer::new(seed.to_vec(), Network::Bitcoin, developer_creds)
            .map_err(|e| AppError::Greenlight(format!("Failed to create signer: {}", e)))?;

        // Recover the node
        let recovery_response = scheduler
            .recover(&signer)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to recover node: {}", e)))?;

        // Return device credentials as bytes
        Ok(DeviceCredentials {
            creds: recovery_response.creds,
        })
    }

    #[allow(dead_code)]
    pub async fn connect_to_node(&self, device_creds: &[u8]) -> Result<()> {
        // Load device credentials from the stored bytes  
        let device = Device::from_bytes(device_creds);
        
        // For now, we just validate that we can create the device
        // A real connection would require the node_id from the signer
        if device.to_bytes().is_empty() {
            return Err(AppError::Greenlight("Invalid device credentials".to_string()));
        }
        
        Ok(())
    }

    pub async fn get_node_info(&self, device_creds: &[u8]) -> Result<Value> {
        // Load device credentials from stored bytes
        let device = Device::from_bytes(device_creds);
        
        // Load developer credentials for scheduler creation
        let developer_cert = fs::read(&self.config.gl_cert_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read cert file: {}", e)))?;
        
        let developer_key = fs::read(&self.config.gl_key_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read key file: {}", e)))?;
        
        let developer_creds = Nobody {
            cert: developer_cert,
            key: developer_key,
            ..Nobody::default()
        };

        // Create scheduler and authenticate with device credentials
        let scheduler = Scheduler::new(Network::Bitcoin, developer_creds)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to create scheduler: {}", e)))?;

        let scheduler = scheduler
            .authenticate(device)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to authenticate: {}", e)))?;

        // Get node client
        let mut node: ClnClient = scheduler
            .node()
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to get node: {}", e)))?;

        // Get node info
        let getinfo_response = node
            .getinfo(gl_client::pb::cln::GetinfoRequest {})
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to get node info: {}", e)))?;

        let info = getinfo_response.get_ref();
        Ok(serde_json::json!({
            "node_id": hex::encode(&info.id),
            "alias": info.alias,
            "color": hex::encode(&info.color),
            "num_peers": info.num_peers,
            "num_pending_channels": info.num_pending_channels,
            "num_active_channels": info.num_active_channels,
            "num_inactive_channels": info.num_inactive_channels,
            "blockheight": info.blockheight,
            "network": info.network,
            "fees_collected_msat": info.fees_collected_msat.as_ref().map(|amt| amt.msat).unwrap_or(0)
        }))
    }

    pub async fn get_balance(&self, device_creds: &[u8]) -> Result<Value> {
        // Load device credentials from stored bytes
        let device = Device::from_bytes(device_creds);
        
        // Load developer credentials for scheduler creation
        let developer_cert = fs::read(&self.config.gl_cert_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read cert file: {}", e)))?;
        
        let developer_key = fs::read(&self.config.gl_key_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read key file: {}", e)))?;
        
        let developer_creds = Nobody {
            cert: developer_cert,
            key: developer_key,
            ..Nobody::default()
        };

        // Create scheduler and authenticate
        let scheduler = Scheduler::new(Network::Bitcoin, developer_creds)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to create scheduler: {}", e)))?;

        let scheduler = scheduler
            .authenticate(device)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to authenticate: {}", e)))?;

        // Get node client
        let mut node: ClnClient = scheduler
            .node()
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to get node: {}", e)))?;

        // Get balance via listfunds
        let funds_response = node
            .list_funds(gl_client::pb::cln::ListfundsRequest {
                spent: Some(false),
            })
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to get balance: {}", e)))?;

        let funds = funds_response.get_ref();
        
        // Calculate onchain balance
        let mut onchain_balance_msat = 0u64;
        for output in &funds.outputs {
            if output.status == 1 { // confirmed
                onchain_balance_msat += output.amount_msat.as_ref().map(|amt| amt.msat).unwrap_or(0);
            }
        }

        // Calculate channel balance
        let mut channel_balance_msat = 0u64;
        for channel in &funds.channels {
            channel_balance_msat += channel.our_amount_msat.as_ref().map(|amt| amt.msat).unwrap_or(0);
        }

        Ok(serde_json::json!({
            "onchain_balance_sat": onchain_balance_msat / 1000,
            "onchain_balance_msat": onchain_balance_msat,
            "channel_balance_sat": channel_balance_msat / 1000,
            "channel_balance_msat": channel_balance_msat,
            "total_balance_sat": (onchain_balance_msat + channel_balance_msat) / 1000,
            "total_balance_msat": onchain_balance_msat + channel_balance_msat
        }))
    }

    pub async fn create_offer(&self, device_creds: &[u8], request: CreateOfferRequest) -> Result<Value> {
        // Load device credentials from stored bytes
        let device = Device::from_bytes(device_creds);
        
        // Load developer credentials for scheduler creation
        let developer_cert = fs::read(&self.config.gl_cert_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read cert file: {}", e)))?;
        
        let developer_key = fs::read(&self.config.gl_key_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read key file: {}", e)))?;
        
        let developer_creds = Nobody {
            cert: developer_cert,
            key: developer_key,
            ..Nobody::default()
        };

        // Create scheduler and authenticate
        let scheduler = Scheduler::new(Network::Bitcoin, developer_creds)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to create scheduler: {}", e)))?;

        let scheduler = scheduler
            .authenticate(device)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to authenticate: {}", e)))?;

        // Get node client
        let mut node: ClnClient = scheduler
            .node()
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to get node: {}", e)))?;

        // Create offer request following the working example
        let offer_request = OfferRequest {
            amount: if let Some(amount) = request.amount_msat {
                if amount > 0 {
                    format!("{}msat", amount)
                } else {
                    "any".to_string()
                }
            } else {
                "any".to_string()
            },
            description: Some(request.description.clone()),
            issuer: Some("".to_string()),
            label: Some(format!("offer_{}", chrono::Utc::now().timestamp())),
            absolute_expiry: None,
            recurrence_base: None,
            recurrence_paywindow: None,
            recurrence_limit: None,
            single_use: None,
            quantity_max: Some(0),
            recurrence: None,
            recurrence_start_any_period: None,
        };

        // Create the offer
        let offer_response = node
            .offer(offer_request)
            .await
            .map_err(|e| AppError::Greenlight(format!("Failed to create offer: {}", e)))?;

        let offer = offer_response.get_ref();
        Ok(serde_json::json!({
            "bolt12": offer.bolt12,
            "offer_id": hex::encode(&offer.offer_id),
            "description": request.description,
            "amount_msat": request.amount_msat,
            "active": !offer.used
        }))
    }

    #[allow(dead_code)]
    pub fn load_credentials(&self) -> Result<Device> {
        let cert_path = &self.config.gl_cert_path;
        let key_path = &self.config.gl_key_path;

        let cert = fs::read(cert_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read cert file: {}", e)))?;
        
        let key = fs::read(key_path)
            .map_err(|e| AppError::Greenlight(format!("Failed to read key file: {}", e)))?;

        // Create device from cert and key with empty rune (for initial connection)
        Ok(Device::with(cert, key, ""))
    }
}
