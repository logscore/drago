import { env } from '$env/dynamic/public';
import { jwtClient } from 'better-auth/client/plugins';
import { createAuthClient } from 'better-auth/svelte';

export const authClient = createAuthClient({
	/** The base URL of the server (optional if you're using the same domain) */
	baseURL: env.PUBLIC_FRONTEND_URL,
	plugins: [jwtClient()]
});
