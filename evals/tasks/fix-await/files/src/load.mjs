export async function loadAll(loaders) {
  return loaders.map((load) => load());
}
