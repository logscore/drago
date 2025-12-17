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
			return Response.redirect(new URL('/auth', event.url), 302);
		}
	}

	if (event.url.pathname.startsWith('/device')) {
		const session = await auth.api.getSession({
			headers: event.request.headers
		});

		if (!session) {
			return Response.redirect(new URL(`/auth?redirect_url=/device`, event.url), 302);
		}
	}

	if (event.url.pathname.startsWith('/auth')) {
		const session = await auth.api.getSession({
			headers: event.request.headers
		});

		if (session) {
			return Response.redirect(new URL('/dashboard', event.url), 302);
		}
	}

	return svelteKitHandler({ event, resolve, auth, building });
}
