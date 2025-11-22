import { auth } from '$lib/auth/auth'; // path to your auth file
import { svelteKitHandler } from 'better-auth/svelte-kit';
import { building } from '$app/environment';

export async function handle({ event, resolve }) {
	// Auth middleware
	if (event.url.pathname.startsWith('/dashboard')) {
		const session = await auth.api.getSession({
			headers: event.request.headers
		});

		if (!session) {
			return Response.redirect(new URL('/', event.url), 302);
		}
	}

	return svelteKitHandler({ event, resolve, auth, building });
}
