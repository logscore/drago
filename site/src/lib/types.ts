export type ZoneRecordData = [Zone, DnsRecord[]][];

export interface Zone {
	id: string;
	name: string;
}

export interface DnsRecord {
	id: string;
	name: string;
	content: string;
	ttl: number;
	type: string;
	proxied: number;
}

export interface AccessToken {
	id: string;
	name: string;
}

export interface ApiKey {
	id: string;
	name: string;
}

export interface AddRecordData {
	name: string;
	content: string;
	type: string;
	ttl: number;
	proxied: number;
}

export interface AddAccessToken {
	name: string;
	value: string;
}
