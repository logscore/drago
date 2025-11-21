type ZoneRecordData = [Zone, DnsRecord[]][];

interface Zone {
	id: string;
	name: string;
	status?: string;
}

interface DnsRecord {
	id: string;
	name: string;
	content: string;
	ttl: number;
	type: string;
	proxied: number;
}

interface AccessKey {
	id: string;
	name: string;
	status?: string;
}

interface AddRecordData{
  name: string,
  content: string,
  type: string,
  ttl: number,
  proxied: number,
}
