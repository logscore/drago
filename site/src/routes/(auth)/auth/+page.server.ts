// src/routes/auth/+page.server.ts
import { auth } from '$lib/auth/auth';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ request, url }) => {
	const session = await auth.api.getSession({
		headers: request.headers
	});

	const redirectUrl = url.searchParams.get('redirect');

	// ONLY redirect here if they are already logged in
	if (session?.user) {
		throw redirect(302, redirectUrl || '/dashboard');
	}

	// Pass the redirectUrl to the client so it's available in `data`
	return {
		redirectUrl
	};
};
