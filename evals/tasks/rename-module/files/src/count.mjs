import { listUsers } from './users.mjs';

export const count = () => listUsers().length;
