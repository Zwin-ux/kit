import { orders } from './orders.mjs';

export const totalItems = () => orders.reduce((n, o) => n + o.qty, 0);
