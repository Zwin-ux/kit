export function shout(args) {
  return args.filter((a) => !a.startsWith('--')).join(' ');
}
