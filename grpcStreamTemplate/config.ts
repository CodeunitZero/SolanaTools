import dotenv from 'dotenv';

dotenv.config();

export const CONFIG = {
    RPC_URL: process.env.RPC_URL ?? "",
    GRPC_URL: process.env.GRPC_URL ?? "",
    PUMPFUN_BONDING: '6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P',
    RAYDIUM_LAUNCHPAD: 'LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj'
};



