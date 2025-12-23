import { auth } from '$lib/auth/auth';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const session = await auth.api.getSession({
		headers: event.request.headers
	});

	const originPath = event.url.pathname;

	// If user is already signed in, redirect to dashboard
	if (!session?.user) {
		throw redirect(302, `/auth?redirect=${originPath}`);
	}

	// Otherwise, let them see the device page
	return {};
};
