import { betterAuth } from 'better-auth';
import { sveltekitCookies } from 'better-auth/svelte-kit';
import { getRequestEvent } from '$app/server';
import { drizzleAdapter } from 'better-auth/adapters/drizzle';
import { db } from '../server/db/index';

export const auth = betterAuth({
	database: drizzleAdapter(db, {
		provider: 'mysql'
	}),
	emailAndPassword: {
		enabled: true,
		autoSignIn: true
	},
	plugins: [sveltekitCookies(getRequestEvent)], // make sure this is the last plugin in the array
	telemetry: { enabled: false }
});
