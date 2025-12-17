import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url }) => {
	const userCode = url.searchParams.get('user_code');

	// Redirect if no user code
	if (!userCode) {
		redirect(302, '/device');
	}

	return {
		userCode
	};
};
