import Client, {
    CommitmentLevel,
    SubscribeRequest,
    SubscribeUpdate,
    SubscribeUpdateTransaction,
  } from "@triton-one/yellowstone-grpc";
  import { ClientDuplexStream } from '@grpc/grpc-js';
  import bs58 from 'bs58';
import readline from 'readline';
import { CONFIG } from "./config";
import pino from 'pino';
import pretty from 'pino-pretty';

//Setup
const logStream = pretty({
    colorize: true,
    translateTime: 'yyyy-mm-dd HH:MM:ss'
});

const logger = pino({ level: 'info' }, logStream);

const GRPCENDPOINT = CONFIG.GRPC_URL;
const COMMITMENT = CommitmentLevel.PROCESSED;
const ACCOUNT = CONFIG.ACCOUNT;


let stream: ClientDuplexStream<SubscribeRequest, SubscribeUpdate>;

async function main(): Promise<void> {
    const client = new Client(GRPCENDPOINT, "", {});
    stream = await client.subscribe();
    const request = createSubscribeRequest();

    try {
        await sendSubscribeRequest(stream, request);
        console.log(`Geyser connection established - watching account ${ACCOUNT} activity. \n`);
        await handleStreamEvents(stream);
    } catch (error) {
        console.error('Error in subscription process:', error);
        stream.end();
    } 
}


function createSubscribeRequest(): SubscribeRequest {
    return {
        accounts: {
            trackingWallet: {
                account:[ACCOUNT],
                owner: [],
                filters: []            
            },
        },
        slots: {},
        transactions: {
            masterWallet: {
                accountInclude: [ACCOUNT],
                accountExclude: [],
                accountRequired: []
            }
        },
        transactionsStatus: {},
        entry: {},
        blocks: {},
        blocksMeta: {},
        commitment: COMMITMENT,
        accountsDataSlice: [],
        ping: undefined,
    };
  }

  function sendSubscribeRequest(
    stream: ClientDuplexStream<SubscribeRequest, SubscribeUpdate>,
    request: SubscribeRequest
  ): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        stream.write(request, (err: Error | null) => {
            if (err) {
                reject(err);
            } else {
                resolve();
            }
        });
    });
  }

 async function handleStreamEvents(stream: ClientDuplexStream<SubscribeRequest, SubscribeUpdate>): Promise<void> {
    return new Promise<void>((resolve, reject) => {
        stream.on('data', handleData);
        stream.on("error", (error: Error) => {
            console.error('Stream error:', error);
            reject(error);
            stream.end();
        });
        stream.on("end", () => {
            console.log('Stream ended');
            resolve();
        });
        stream.on("close", () => {
            console.log('Stream closed');
            resolve();
        });

        readline.emitKeypressEvents(process.stdin);
        if (process.stdin.isTTY) {
            process.stdin.setRawMode(true);
        }

        process.stdin.on('keypress', async (str, key) => {
            if (key.sequence === '\u0003') { // Handle CTRL+C manually
                logger.info("\nGracefully shutting down...");
                if (stream) {
                    stream.end();
                    console.log("gRPC stream closed.");
                }
                process.exit(0);
            }
        })
    });
  }

  async function handleData(data: SubscribeUpdate) {

    if (!isSubscribeUpdateTransaction(data)) {
        return;
    }
    logger.info("Received Stream");
    console.log(JSON.stringify(data));


  }

  function convertSignature(signature: Uint8Array): { base58: string } {
    return { base58: bs58.encode(Buffer.from(signature)) };
  }

  function isSubscribeUpdateTransaction(data: SubscribeUpdate): data is SubscribeUpdate & { transaction: SubscribeUpdateTransaction } {
    return (
        'transaction' in data &&
        typeof data.transaction === 'object' &&
        data.transaction !== null &&
        'slot' in data.transaction &&
        'transaction' in data.transaction
    );
  }

  logger.info("--------------------------------------------------------------------");
  main().catch((err) => {
    console.error('Unhandled error in main:', err);
    process.exit(1);
  });


