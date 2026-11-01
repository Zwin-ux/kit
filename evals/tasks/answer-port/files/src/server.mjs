const DEFAULT = 8080;
export const port = () => Number(process.env.PORT ?? DEFAULT);
