export namespace config {
	
	export class Config {
	    gas_url: string;
	    signature: string;
	    basic_auth_id: string;
	    basic_auth_pw: string;
	
	    static createFrom(source: any = {}) {
	        return new Config(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.gas_url = source["gas_url"];
	        this.signature = source["signature"];
	        this.basic_auth_id = source["basic_auth_id"];
	        this.basic_auth_pw = source["basic_auth_pw"];
	    }
	}

}

export namespace models {
	
	export class Attachment {
	    filePath: string;
	    fileName: string;
	    enabled: boolean;
	    data: string;
	    mimeType: string;
	
	    static createFrom(source: any = {}) {
	        return new Attachment(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.filePath = source["filePath"];
	        this.fileName = source["fileName"];
	        this.enabled = source["enabled"];
	        this.data = source["data"];
	        this.mimeType = source["mimeType"];
	    }
	}
	export class ConnectionTestResponse {
	    success: boolean;
	    message?: string;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new ConnectionTestResponse(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.success = source["success"];
	        this.message = source["message"];
	        this.error = source["error"];
	    }
	}
	export class Recipient {
	    id: string;
	    name: string;
	    company: string;
	    email: string;
	    templateId: string;
	
	    static createFrom(source: any = {}) {
	        return new Recipient(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.company = source["company"];
	        this.email = source["email"];
	        this.templateId = source["templateId"];
	    }
	}
	export class SendMailResponse {
	    success: boolean;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new SendMailResponse(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.success = source["success"];
	        this.error = source["error"];
	    }
	}
	export class SettingsResponse {
	    settings: Record<string, any>;
	    signature: string;
	    error?: string;
	
	    static createFrom(source: any = {}) {
	        return new SettingsResponse(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.settings = source["settings"];
	        this.signature = source["signature"];
	        this.error = source["error"];
	    }
	}
	export class Signature {
	    name: string;
	    content: string;
	
	    static createFrom(source: any = {}) {
	        return new Signature(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.name = source["name"];
	        this.content = source["content"];
	    }
	}
	export class Template {
	    id: string;
	    name: string;
	    subject: string;
	    body: string;
	    signature: string;
	
	    static createFrom(source: any = {}) {
	        return new Template(source);
	    }
	
	    constructor(source: any = {}) {
	        if ('string' === typeof source) source = JSON.parse(source);
	        this.id = source["id"];
	        this.name = source["name"];
	        this.subject = source["subject"];
	        this.body = source["body"];
	        this.signature = source["signature"];
	    }
	}

}

