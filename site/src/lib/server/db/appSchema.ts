import {
	mysqlTable,
	varchar,
	text,
	timestamp,
	int,
	index,
	varbinary,
	boolean
} from 'drizzle-orm/mysql-core';
import { user } from './authSchema';

// ---------------------------
// DNS API Tokens
// ---------------------------
export const dnsToken = mysqlTable('dns_token', {
	id: varchar('id', { length: 225 }).primaryKey(),
	name: varchar('name', { length: 225 }).notNull(),
	userId: varchar('user_id', { length: 36 })
		.notNull()
		.references(() => user.id, { onDelete: 'cascade' }),
	nonce: varbinary('nonce', { length: 12 }).notNull(), // 12-byte AES-GCM IV
	tokenEncrypted: varbinary('token_encrypted', { length: 1024 }).notNull(), // ciphertext
	tag: varbinary('tag', { length: 16 }).notNull(),
	createdOn: timestamp('created_on').defaultNow().notNull(),
	updatedOn: timestamp('updated_on')
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
		name: varchar('name', { length: 255 }).notNull(),
		prefixId: varchar('prefix_id', { length: 20 }).notNull().unique(), // Indexed lookup
		keyHash: varchar('key_hash', { length: 97 }).notNull(),
		// Each API key can control a single DNS record. This is how we identify which record to update when a client syncs to us.
		dns_record_id: varchar('dns_record_id', { length: 255 })
			.references(() => dnsRecord.id)
			.unique()
			.notNull(),
		lastUsed: timestamp('last_used'),
		createdOn: timestamp('created_on').defaultNow().notNull(),
		updatedOn: timestamp('updated_on')
			.defaultNow()
			.$onUpdate(() => new Date())
			.notNull()
	},
	(t) => ({
		userIdx: index('idx_client_api_key_user_id').on(t.userId),
		prefixIdx: index('idx_client_api_key_prefix').on(t.prefixId)
	})
);

// ---------------------------
// DNS Records
// ---------------------------
export const dnsRecord = mysqlTable(
	'dns_record',
	{
		// This value
		id: varchar('id', { length: 255 }).primaryKey(), // DNS provider's internal ID
		userId: varchar('user_id', { length: 36 })
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		zoneId: varchar('zone_id', { length: 255 })
			.notNull()
			.references(() => dnsZone.id, { onDelete: 'cascade' }),
		recordName: varchar('record_name', { length: 255 }).notNull(),
		recordType: varchar('record_type', { length: 16 }).notNull(), // e.g. A, AAAA, CNAME, TXT
		content: text('content').notNull(), // IP, domain target, TXT value, etc.
		ttl: int('ttl').default(3600).notNull(),
		proxied: boolean('proxied').default(false).notNull(),
		lastSyncedOn: timestamp('last_synced_on')
			.defaultNow()
			.$onUpdate(() => new Date())
			.notNull()
	},
	(t) => ({
		userIdx: index('idx_dns_record_user_id').on(t.userId),
		recordNameIdx: index('idx_dns_record_record_name').on(t.recordName),
		typeIdx: index('idx_dns_record_type').on(t.recordType)
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
		lastSyncedOn: timestamp('last_synced_on')
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
