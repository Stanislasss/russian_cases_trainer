/** Where to go after login: only in-app paths, never an external URL from `?next=`. */
export function safeNext(next: string | null): string {
  return next?.startsWith('/') && !next.startsWith('//') ? next : '/'
}
