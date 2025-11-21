interface ZoneRecordData {
	zones: Zone[];
	dns_records: [DnsRecord[]];
}

interface Zone {
	id: string;
	name: string;
	status?: string;
}

interface DnsRecord {
	id: string;
	name: string;
	content: string;
}

interface AccessKey {
	id: string;
	name: string;
	status?: string;
}
