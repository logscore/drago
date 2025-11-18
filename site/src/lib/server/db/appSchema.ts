import { mysqlTable, varchar, text, timestamp, int, index } from 'drizzle-orm/mysql-core';
import { user } from './authSchema'; // assuming auth schema file exports user

// ---------------------------
// DNS API Tokens
// ---------------------------
export const dnsToken = mysqlTable('dns_token', {
	id: varchar('id', { length: 36 }).primaryKey(),
	userId: varchar('user_id', { length: 36 })
		.notNull()
		.references(() => user.id, { onDelete: 'cascade' }),
	tokenEncrypted: text('token_encrypted').notNull(),
	accountEmail: varchar('account_email', { length: 255 }),
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
		id: varchar('id', { length: 36 }).primaryKey(),
		userId: varchar('user_id', { length: 36 })
			.notNull()
			.references(() => user.id, { onDelete: 'cascade' }),
		recordName: varchar('record_name', { length: 255 }).notNull(),
		zoneId: varchar('zone_id', { length: 255 }).notNull(),
		recordId: varchar('record_id', { length: 255 }).notNull(),
		currentIp: varchar('current_ip', { length: 45 }),
		createdAt: timestamp('created_at', { fsp: 3 }).defaultNow().notNull(),
		updatedAt: timestamp('updated_at', { fsp: 3 })
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
