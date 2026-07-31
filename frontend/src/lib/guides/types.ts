export type GuideSlug = 'students' | 'faculty' | 'admin';

export interface GuideStep {
	/** What to do, in the imperative. */
	action: string;
	/** What happens, or what to watch for. */
	detail?: string;
	/** The exact button, tab, or menu item to click. */
	click?: string;
}

export interface GuideSection {
	/** Shown as the bold numbered header. */
	title: string;
	/** One or two lines on why this section exists. */
	summary: string;
	steps: GuideStep[];
	/** Pulled out as a callout under the steps. */
	note?: string;
}

export interface Guide {
	slug: GuideSlug;
	/** Sidebar and tab label. */
	label: string;
	title: string;
	/** Who the guide is written for. */
	audience: string;
	intro: string;
	sections: GuideSection[];
}
