use bytes::Bytes;
use clap::Parser;
use rand::{thread_rng, Rng};
use self_encryption::MIN_ENCRYPTABLE_BYTES;
use sn_client::{Client, ClientConfig, Error};
use sn_interface::{
    messaging::{
        data::{DataQuery, DataQueryVariant, QueryResponse, ServiceMsg},
        WireMsg,
    },
    types::{Chunk, ChunkAddress},
};
use std::{
    fs::File,
    io::{ErrorKind, Write},
};
use xor_name::XorName;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
#[clap(long_about = None)]
struct Args {
    /// Path of file to use
    #[clap()]
    file: String,

    /// Upload the file to the network before querying
    #[clap(short, long)]
    upload: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize

    let (args, bytes) = init()?;
    let client = client().await?;

    // Print out chunk information of file

    let (data_map_xor, chunks) = client.chunk_bytes(bytes.clone())?;
    println!("DataMap XOR: {}", data_map_xor);
    for chunk in &chunks {
        println!(
            "Chunk (length: {}) XOR: {}",
            chunk.value().len(),
            chunk.address().0
        );
    }

    // Upload file to network

    if args.upload {
        let xor = client.upload(bytes).await?;
        println!("Uploaded file with DataMap XOR: {}", xor);

        assert_eq!(data_map_xor, xor);
    }

    // Query adults for chunks

    let mut error_occurred = false;
    for chunk in chunks {
        for i in 0..3 {
            if let Err(e) = query_chunk(&client, i, chunk.address().0).await {
                error_occurred = true;
                eprintln!(
                    "Error for Chunk({}) at adult `{}`: {:?}",
                    chunk.address().0,
                    i,
                    e
                );
            }
        }
    }

    if error_occurred {
        return Err("A chunk was not succesfully returned from an adult").map_err(|e| e.into());
    }

    Ok(())
}

async fn query_chunk(client: &Client, adult: usize, address: XorName) -> Result<Chunk> {
    let query_response = send_query(client, address, adult).await?;
    match query_response {
        QueryResponse::GetChunk(result) => result.map_err(|e| e.into()),
        response => Err(Error::UnexpectedQueryResponse(response)).map_err(|e| e.into()),
    }
}

async fn send_query(
    client: &Client,
    address: XorName,
    adult_index: usize,
) -> Result<QueryResponse> {
    let query = DataQuery {
        adult_index,
        variant: DataQueryVariant::GetChunk(ChunkAddress(address)),
    };

    let client_pk = client.public_key();
    let msg = ServiceMsg::Query(query.clone());
    let serialised_query = WireMsg::serialize_msg_payload(&msg)?;
    let signature = client.keypair().sign(&serialised_query);

    Ok(client
        .send_signed_query(
            query.clone(),
            client_pk,
            serialised_query.clone(),
            signature.clone(),
        )
        .await?
        .response)
}

/// Parse arguments and setup file/bytes to use
fn init() -> Result<(Args, Bytes)> {
    let args = Args::parse();

    // Read bytes from file
    let bytes = match std::fs::read(&args.file) {
        Ok(contents) => Bytes::from(contents),
        // If file is not found, generate file and put it at file path
        Err(err) if err.kind() == ErrorKind::NotFound => {
            let bytes = self_encryptable_file();

            let mut f = File::create(&args.file)?;
            f.write_all(&bytes)?;

            println!("Wrote random data file to \"{}\"", &args.file);

            bytes
        }
        Err(err) => Err(err)?,
    };

    Ok((args, bytes))
}

/// Generate random bytes with minimal self-encryption size
fn self_encryptable_file() -> Bytes {
    let mut file = [0u8; MIN_ENCRYPTABLE_BYTES];
    thread_rng().fill(&mut file[..]);

    Bytes::copy_from_slice(&file)
}

/// Construct Client from configuration files in default location
async fn client() -> Result<Client> {
    let root_dir = tempfile::tempdir()?;

    let config = ClientConfig::new(Some(root_dir.path()), None, None, None, None, None).await;

    Ok(Client::new(config, None, None).await?)
}
