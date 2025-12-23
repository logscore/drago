import { auth } from '$lib/auth/auth';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async (event) => {
	const session = await auth.api.getSession({
		headers: event.request.headers
	});

	// If user isnt already signed in, redirect to auth
	if (!session?.user) {
		throw redirect(302, '/auth');
	}

	// Otherwise, let them see the dashboard page
	return {};
};
