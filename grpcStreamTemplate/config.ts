import dotenv from 'dotenv';

dotenv.config();

export const CONFIG = {
    RPC_URL: process.env.RPC_URL ?? "",
    GRPC_URL: process.env.GRPC_URL ?? "",
    ACCOUNT: '675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8',
};



