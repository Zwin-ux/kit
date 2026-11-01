import { listUsers } from './users.mjs';

export const report = () => listUsers().join(', ');
