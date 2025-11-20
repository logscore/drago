import {
	mysqlTable,
	varchar,
	text,
	timestamp,
	int,
	index,
	varbinary
} from 'drizzle-orm/mysql-core';
import { user } from './authSchema'; // assuming auth schema file exports user

// ---------------------------
// DNS API Tokens
// ---------------------------
export const dnsToken = mysqlTable('dns_token', {
	id: varchar('id', { length: 225 }).primaryKey(),
	userId: varchar('user_id', { length: 36 })
		.notNull()
		.references(() => user.id, { onDelete: 'cascade' }),
	nonce: varbinary('nonce', { length: 12 }).notNull(), // 12-byte AES-GCM IV
	tokenEncrypted: varbinary('token_encrypted', { length: 1024 }).notNull(), // ciphertext
	tag: varbinary('tag', { length: 16 }).notNull(),
	createdAt: timestamp('created_at', { fsp: 3 }).defaultNow().notNull(),
	updatedAt: timestamp('updated_at', { fsp: 3 })
		.defaultNow()
		.$onUpdate(() => new Date())
		.notNull()
});

// ---------------------------
// Client API Keys
// ---------------------------
export const apiKeys = mysqlTable(
	'api_keys',
	{
		id: varchar('id', { length: 36 }).primaryKey(),
		userId: varchar('user_id', { length: 36 })
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		keyHash: varchar('key_hash', { length: 64 }).notNull().unique(),
		lastUsed: timestamp('last_used', { fsp: 3 }),
		createdAt: timestamp('created_at', { fsp: 3 }).defaultNow().notNull(),
		updatedAt: timestamp('updated_at', { fsp: 3 })
			.defaultNow()
			.$onUpdate(() => new Date())
			.notNull()
	},
	(t) => ({
		userIdx: index('idx_client_api_key_user_id').on(t.userId)
	})
);

// ---------------------------
// DNS Records
// ---------------------------
export const dnsRecord = mysqlTable(
	'dns_record',
	{
		id: varchar('id', { length: 255 }).primaryKey(), // DNS providers internal ID
		userId: varchar('user_id', { length: 36 })
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		recordName: varchar('record_name', { length: 255 }).notNull(),
		zoneId: varchar('zone_id', { length: 255 })
			.notNull()
			.references(() => dnsZone.id, { onDelete: 'cascade' }), // DNS providers internal ID
		content: text('content'), // This is the IP, Domain name, or whatever else.
		createdAt: timestamp('created_at', { fsp: 3 }).defaultNow().notNull(),
		lastSyncedAt: timestamp('last_synced_at')
			.defaultNow()
			.$onUpdate(() => new Date())
			.notNull()
	},
	(t) => ({
		userIdx: index('idx_dns_record_user_id').on(t.userId),
		recordNameIdx: index('idx_dns_record_record_name').on(t.recordName)
	})
);

// ---------------------------
// DNS Zones
// ---------------------------
export const dnsZone = mysqlTable(
	'dns_zone',
	{
		id: varchar('id', { length: 255 }).primaryKey(), // DNS provider internal zone ID
		userId: varchar('user_id', { length: 36 })
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		tokenId: varchar('token_id', { length: 225 })
			.notNull()
			.references(() => dnsToken.id, { onDelete: 'cascade' }),
		zoneName: varchar('zone_name', { length: 255 }).notNull(),
		lastSyncedAt: timestamp('last_synced_at')
			.defaultNow()
			.$onUpdate(() => new Date())
			.notNull(),
		meta: text('meta') // store JSON (for nameservers, plan, status, etc.)
	},
	(t) => ({
		userIdx: index('idx_dns_zone_user_id').on(t.userId),
		tokenIdx: index('idx_dns_zone_token_id').on(t.tokenId),
		zoneNameIdx: index('idx_dns_zone_zone_name').on(t.zoneName)
	})
);
