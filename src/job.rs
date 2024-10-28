use std::fmt::Display;

use chrono::{DateTime, Utc};
use colored::{ColoredString, Colorize as _};
use serde::{Deserialize, Serialize};
use url::Url;

/// A discovered job posting.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Job {
    /// The time when the job was first found.
    pub timestamp: DateTime<Utc>,
    /// The name of the source where the job was found.
    pub source: String,
    /// The name of the company offering the job.
    pub company: String,
    /// The URL to the job page.
    pub url: Url,
    /// The job title.
    pub title: String,
    /// The job level (entry, mid, senior, etc.).
    pub level: JobLevel,
    /// The job specialty (graphics, audio, AI, etc.).
    pub specialty: Option<JobSpecialty>,
    /// The job discipline (programmer, artist, writer, etc.).
    pub discipline: JobDiscipline,
    /// True if the job is an application drop box, not a real opening.
    pub is_general_application: bool,
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.title)
    }
}

impl Job {
    pub fn new(
        source: impl Into<String>,
        company: impl Into<String>,
        url: impl Into<Url>,
        title: impl Into<String>,
    ) -> Self {
        let title = title.into();
        let norm = normalized(&title);

        Self {
            timestamp: Utc::now(),
            source: source.into(),
            company: company.into(),
            url: url.into(),
            title,
            level: parse_level(&norm),
            specialty: parse_specialty(&norm),
            discipline: parse_discipline(&norm),
            is_general_application: parse_is_general_application(&norm),
        }
    }

    pub fn reparse(&mut self) {
        let norm = normalized(&self.title);
        self.level = parse_level(&norm);
        self.specialty = parse_specialty(&norm);
        self.discipline = parse_discipline(&norm);
        self.is_general_application = parse_is_general_application(&norm);
    }

    // TODO: Load preferences from a config file.
    pub fn score(&self) -> i32 {
        let mut score = 0;

        if self.is_general_application {
            score -= 10;
        }
        score += match self.level {
            JobLevel::Intern => -1000,
            JobLevel::Entry => 10,
            JobLevel::Mid => 0,
            JobLevel::Senior => -500,
            JobLevel::Lead => -1000,
        };
        score += match self.discipline {
            JobDiscipline::Programmer => 100,
            JobDiscipline::Designer => -105,
            JobDiscipline::Artist => -105,
            JobDiscipline::Writer => -110,
            JobDiscipline::Composer => -110,
            JobDiscipline::Tester => -125,
            JobDiscipline::Manager => -150,
            JobDiscipline::Other => -110,
        };
        score += match self.specialty {
            Some(JobSpecialty::Gameplay) => 100,
            Some(JobSpecialty::Graphics) => 1,
            Some(JobSpecialty::Engine) => 1,
            Some(JobSpecialty::Physics) => -5,
            Some(JobSpecialty::Animation) => -100,
            Some(JobSpecialty::Ai) => -100,
            Some(JobSpecialty::Audio) => -110,
            Some(JobSpecialty::Ui) => -120,
            Some(JobSpecialty::Network) => -150,
            Some(JobSpecialty::Automation) => -150,
            Some(JobSpecialty::Web) => -150,
            None => 0,
        };

        10 * score
    }

    pub(crate) fn prefix(&self) -> ColoredString {
        if self.score() > 0 {
            "[!] ".bold().green()
        } else {
            "".into()
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum JobLevel {
    Intern,
    Entry,
    Mid,
    Senior,
    Lead,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum JobSpecialty {
    Gameplay,
    Graphics,
    Engine,
    Physics,
    Animation,
    Ai,
    Audio,
    Ui,
    Network,
    Automation,
    Web,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub enum JobDiscipline {
    Programmer,
    Designer,
    Artist,
    Writer,
    Composer,
    Tester,
    Manager,
    Other,
}

fn normalized(s: impl AsRef<str>) -> String {
    s.as_ref()
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

macro_rules! re {
    ($name:ident, $($e:expr),* $(,)?) => {
        static $name: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(
            || regex::Regex::new(concat!($($e),*)).unwrap(),
        );
    };
}

fn parse_level(norm: &str) -> JobLevel {
    re!(
        INTERN_RE,
        r"\b(intern(ship)?|co ?op|(under)?grad(uate)?|thesis)\b",
    );
    re!(ENTRY_RE, r"\b(entry|associate|junior|jr)\b");
    re!(MID_RE, r"\b(mid|executive assistant)\b");
    re!(
        SENIOR_RE,
        r"\b(senior|sn?r|expert|advanced?|principal|staff)\b",
    );
    re!(
        LEAD_RE,
        r"\b(lead|director|president|executive|head|architect)\b",
    );

    if INTERN_RE.is_match(norm) {
        JobLevel::Intern
    } else if ENTRY_RE.is_match(norm) {
        JobLevel::Entry
    } else if MID_RE.is_match(norm) {
        JobLevel::Mid
    } else if SENIOR_RE.is_match(norm) {
        JobLevel::Senior
    } else if LEAD_RE.is_match(norm) {
        JobLevel::Lead
    } else {
        JobLevel::Mid
    }
}

fn parse_specialty(norm: &str) -> Option<JobSpecialty> {
    re!(
        AUTOMATION_RE,
        r"\b(automation|build|release|security|devops?|test(ing)?",
        r"|sdet|reliability|sre|(platforms?|data|migration) engineer(ing)?)\b",
    );
    re!(WEB_RE, r"\b(web|front ?end)\b");
    re!(
        GRAPHICS_RE,
        r"\b(graphics|rendering|art|technical artist)\b",
    );
    re!(ANIMATION_RE, r"\b(animation)\b");
    re!(PHYSICS_RE, r"\b(physics)\b");
    re!(AUDIO_RE, r"\b(audio)\b");
    re!(AI_RE, r"\b(computer vision|machine learning)\b");
    re!(UI_RE, r"\b(ui|ux|user interface|user experience)\b");
    re!(NETWORK_RE, r"\b(network|server|services?|backend)\b");
    re!(ENGINE_RE, r"\b(engine programmer|tools|technology)\b");
    re!(GAMEPLAY_RE, r"\b(gameplay|game|unity|unreal)\b");
    re!(SOFT_AI_RE, r"\b(ai)\b");

    if AUTOMATION_RE.is_match(norm) {
        Some(JobSpecialty::Automation)
    } else if WEB_RE.is_match(norm) {
        Some(JobSpecialty::Web)
    } else if GRAPHICS_RE.is_match(norm) {
        Some(JobSpecialty::Graphics)
    } else if ANIMATION_RE.is_match(norm) {
        Some(JobSpecialty::Animation)
    } else if PHYSICS_RE.is_match(norm) {
        Some(JobSpecialty::Physics)
    } else if AUDIO_RE.is_match(norm) {
        Some(JobSpecialty::Audio)
    } else if AI_RE.is_match(norm) {
        Some(JobSpecialty::Ai)
    } else if UI_RE.is_match(norm) {
        Some(JobSpecialty::Ui)
    } else if NETWORK_RE.is_match(norm) {
        Some(JobSpecialty::Network)
    } else if ENGINE_RE.is_match(norm) {
        Some(JobSpecialty::Engine)
    } else if GAMEPLAY_RE.is_match(norm) {
        Some(JobSpecialty::Gameplay)
    } else if SOFT_AI_RE.is_match(norm) {
        Some(JobSpecialty::Ai)
    } else {
        None
    }
}

fn parse_discipline(norm: &str) -> JobDiscipline {
    re!(
        MANAGER_RE,
        r"\b(manager|director|president|coordinator|producer)\b",
    );
    re!(TESTER_RE, r"\b(tester|qa|quality engineer(ing)?)\b");
    re!(
        KNOWN_OTHER_RE,
        r"\b((bi|support|privacy|facility|mechatronics|enterprise solution) engineer(ing)?",
        r"|it|information technology|hr|human resources?|representative)\b",
    );
    re!(
        PROGRAMMER_RE,
        r"\b(programmer|coder|developer|engineer(ing)?|technical artist|swe|sre)\b",
    );
    re!(
        OTHER_RE,
        r"\b(specialist|researcher|scientist|analyst|assistant|responder",
        r"|publishing|marketing|support)\b",
    );
    re!(ARTIST_RE, r"\b(artist|animator|modeler|3d generalist)\b");
    re!(WRITER_RE, r"\b(writer)\b");
    re!(COMPOSER_RE, r"\b(composer)\b");
    re!(DESIGNER_RE, r"\b(designer|architect)\b");
    re!(HEAD_RE, r"\b(lead|head)\b");
    re!(GENERALIST_RE, r"\b(generalist)\b");

    if MANAGER_RE.is_match(norm) {
        JobDiscipline::Manager
    } else if TESTER_RE.is_match(norm) {
        JobDiscipline::Tester
    } else if KNOWN_OTHER_RE.is_match(norm) {
        JobDiscipline::Other
    } else if PROGRAMMER_RE.is_match(norm) {
        JobDiscipline::Programmer
    } else if OTHER_RE.is_match(norm) {
        JobDiscipline::Other
    } else if ARTIST_RE.is_match(norm) {
        JobDiscipline::Artist
    } else if WRITER_RE.is_match(norm) {
        JobDiscipline::Writer
    } else if COMPOSER_RE.is_match(norm) {
        JobDiscipline::Composer
    } else if DESIGNER_RE.is_match(norm) {
        JobDiscipline::Designer
    } else if HEAD_RE.is_match(norm) {
        JobDiscipline::Manager
    } else if GENERALIST_RE.is_match(norm) {
        JobDiscipline::Programmer
    } else {
        JobDiscipline::Other
    }
}

fn parse_is_general_application(norm: &str) -> bool {
    re!(
        GENERAL_APPLICATION_RE,
        r"\b(general application|drop box)\b",
    );

    GENERAL_APPLICATION_RE.is_match(norm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level() {
        for (title, level, _, _) in TEST_CASES {
            assert_eq!(parse_level(&normalized(title)), level, "{}", title);
        }
    }

    #[test]
    fn specialty() {
        for (title, _, specialty, _) in TEST_CASES {
            assert_eq!(parse_specialty(&normalized(title)), specialty, "{}", title);
        }
    }

    #[test]
    fn discipline() {
        for (title, _, _, discipline) in TEST_CASES {
            assert_eq!(
                parse_discipline(&normalized(title)),
                discipline,
                "{}",
                title
            );
        }
    }

    #[test]
    fn is_general_application() {
        for s in ["Engineering Application Drop Box", "General Application"] {
            assert!(parse_is_general_application(&normalized(s)), "{}", s);
        }
    }

    const TEST_CASES: [(&str, JobLevel, Option<JobSpecialty>, JobDiscipline); 93] = [
        (
            "Software Engineer Intern - Automation",
            JobLevel::Intern,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Software Engineer Co-op/Internship (FC) - Summer 2025",
            JobLevel::Intern,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Software Engineer Co-Op (Fall 2025)",
            JobLevel::Intern,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Tools Engineer Co-Op- 4 Month Summer 2025 (Apex Legends)",
            JobLevel::Intern,
            Some(JobSpecialty::Engine),
            JobDiscipline::Programmer,
        ),
        (
            "PhD Software Engineer Intern",
            JobLevel::Intern,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "2K Games Dublin - Publishing Graduate Programme",
            JobLevel::Intern,
            None,
            JobDiscipline::Other,
        ),
        (
            "Intern - World Designer",
            JobLevel::Intern,
            None,
            JobDiscipline::Designer,
        ),
        (
            "Senior Software Development Engineer in Test",
            JobLevel::Senior,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Sr Advanced Online/Network Software Engineer - American Football",
            JobLevel::Senior,
            Some(JobSpecialty::Network),
            JobDiscipline::Programmer,
        ),
        (
            "Principal Game Software Engineer (Apex Legends)",
            JobLevel::Senior,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Senior/Lead C++ Software Engineer (Generalist - Game Modes) - American Football",
            JobLevel::Senior,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Senior/Principal Software Engineer - Cell Lifecycle",
            JobLevel::Senior,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "(Senior) Server Engineer",
            JobLevel::Senior,
            Some(JobSpecialty::Network),
            JobDiscipline::Programmer,
        ),
        (
            "Principal Software Engineer , Graphics | Diablo IV | Albany, NY OR Irvine, CA",
            JobLevel::Senior,
            Some(JobSpecialty::Graphics),
            JobDiscipline::Programmer,
        ),
        (
            "Staff Software Engineer (Build Platforms) - VALORANT, Foundations",
            JobLevel::Senior,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Expert Gameplay Animation Engineer",
            JobLevel::Senior,
            Some(JobSpecialty::Animation),
            JobDiscipline::Programmer,
        ),
        (
            "Expert Backend Engineer",
            JobLevel::Senior,
            Some(JobSpecialty::Network),
            JobDiscipline::Programmer,
        ),
        (
            "Advanced Software Engineer",
            JobLevel::Senior,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "PROGRAMMING - Senior Programmer - General",
            JobLevel::Senior,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Graphics Programmer (Staff/Senior)",
            JobLevel::Senior,
            Some(JobSpecialty::Graphics),
            JobDiscipline::Programmer,
        ),
        (
            "Software Engineer Lead (Live Technical Support)",
            JobLevel::Lead,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Lead Software Engineer - Frostbite",
            JobLevel::Lead,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Unity UI Engineer - Unannounced Project",
            JobLevel::Mid,
            Some(JobSpecialty::Ui),
            JobDiscipline::Programmer,
        ),
        (
            "AI/Gameplay Programmer (Mid / Senior Level)",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "UI Programmer (C++)",
            JobLevel::Mid,
            Some(JobSpecialty::Ui),
            JobDiscipline::Programmer,
        ),
        (
            "Tools Automation Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Animation R&D Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Animation),
            JobDiscipline::Programmer,
        ),
        (
            "UI Tools Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Ui),
            JobDiscipline::Programmer,
        ),
        (
            "Technical Artist",
            JobLevel::Mid,
            Some(JobSpecialty::Graphics),
            JobDiscipline::Programmer,
        ),
        (
            "Physics Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Physics),
            JobDiscipline::Programmer,
        ),
        (
            "Animation Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Animation),
            JobDiscipline::Programmer,
        ),
        (
            "Unreal Automation Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Unreal UI Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Ui),
            JobDiscipline::Programmer,
        ),
        (
            "Engine Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Engine),
            JobDiscipline::Programmer,
        ),
        (
            "Graphics Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Graphics),
            JobDiscipline::Programmer,
        ),
        (
            "Tools Engineer (Retro Studios)",
            JobLevel::Mid,
            Some(JobSpecialty::Engine),
            JobDiscipline::Programmer,
        ),
        (
            "Technology Engineer [Remote Contract] (Retro Studios)",
            JobLevel::Mid,
            Some(JobSpecialty::Engine),
            JobDiscipline::Programmer,
        ),
        (
            "Network Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Network),
            JobDiscipline::Programmer,
        ),
        (
            "Audio Software Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Audio),
            JobDiscipline::Programmer,
        ),
        (
            "Computer Vision Software Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Ai),
            JobDiscipline::Programmer,
        ),
        (
            "Architect (Unreal Engine)",
            JobLevel::Lead,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Designer,
        ),
        (
            "Director of Engineering",
            JobLevel::Lead,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Vice President, Global Services",
            JobLevel::Lead,
            Some(JobSpecialty::Network),
            JobDiscipline::Manager,
        ),
        (
            "UGX -Technical Director",
            JobLevel::Lead,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Technical Lead - Maxis",
            JobLevel::Lead,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Technical Director of Gameplay",
            JobLevel::Lead,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Manager,
        ),
        (
            "Executive Producer",
            JobLevel::Lead,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Head of Infrastructure - Monopoly GO!",
            JobLevel::Lead,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Systems Designer (Senior)",
            JobLevel::Senior,
            None,
            JobDiscipline::Designer,
        ),
        (
            "Expert Gameplay Animator - Infinity Ward",
            JobLevel::Senior,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Artist,
        ),
        (
            "Sr. Manager, Software Engineering - Player Platform SDK",
            JobLevel::Senior,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Sr BI Engineer, Amazon Games",
            JobLevel::Senior,
            None,
            JobDiscipline::Other,
        ),
        (
            "Site Reliability Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Manager, Software Engineering - League of Legends, Hextech Engine",
            JobLevel::Mid,
            None,
            JobDiscipline::Manager,
        ),
        ("Dev QA Tester", JobLevel::Mid, None, JobDiscipline::Tester),
        ("QA Tester", JobLevel::Mid, None, JobDiscipline::Tester),
        (
            "User Experience Researcher, Shared Development Services",
            JobLevel::Mid,
            Some(JobSpecialty::Ui),
            JobDiscipline::Other,
        ),
        (
            "Art Director",
            JobLevel::Lead,
            Some(JobSpecialty::Graphics),
            JobDiscipline::Manager,
        ),
        (
            "Technical Stage Manager",
            JobLevel::Mid,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Associate Manager, Global Social Media Marketing - NBA 2K",
            JobLevel::Entry,
            None,
            JobDiscipline::Manager,
        ),
        ("Data Scientist", JobLevel::Mid, None, JobDiscipline::Other),
        (
            "Platforms Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Application Security Specialist",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Other,
        ),
        (
            "Incident Responder",
            JobLevel::Mid,
            None,
            JobDiscipline::Other,
        ),
        (
            "Gameplay Designer",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Designer,
        ),
        (
            "Data Analytics Tester (3mos) Contract",
            JobLevel::Mid,
            None,
            JobDiscipline::Tester,
        ),
        (
            "Test Manager",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Manager,
        ),
        (
            "People Operations Coordinator",
            JobLevel::Mid,
            None,
            JobDiscipline::Manager,
        ),
        (
            "Environment Artist",
            JobLevel::Mid,
            None,
            JobDiscipline::Artist,
        ),
        (
            "Writer (12 month contract)",
            JobLevel::Mid,
            None,
            JobDiscipline::Writer,
        ),
        (
            "Executive Assistant",
            JobLevel::Mid,
            None,
            JobDiscipline::Other,
        ),
        (
            "Materials Artist, NBA 2K",
            JobLevel::Mid,
            None,
            JobDiscipline::Artist,
        ),
        ("Animator", JobLevel::Mid, None, JobDiscipline::Artist),
        ("Data Analyst 2", JobLevel::Mid, None, JobDiscipline::Other),
        (
            "Application Security Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Software Developer in Test - Gram Games",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Level Designer",
            JobLevel::Mid,
            None,
            JobDiscipline::Designer,
        ),
        (
            "Systems Designer - Sledgehammer Games Toronto",
            JobLevel::Mid,
            None,
            JobDiscipline::Designer,
        ),
        (
            "DevOps Engineer (Kubernetes & Cloud Services)",
            JobLevel::Mid,
            Some(JobSpecialty::Automation),
            JobDiscipline::Programmer,
        ),
        (
            "Machine Learning Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Ai),
            JobDiscipline::Programmer,
        ),
        (
            "Gameplay Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Gameplay Engineer - High Moon Studios",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Gameplay Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Game Programmer",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Software Engineer, Gameplay",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Software Engineer",
            JobLevel::Mid,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Software Development Engineer (Cardset)",
            JobLevel::Mid,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Software Development Engineer (Server Developer)",
            JobLevel::Mid,
            Some(JobSpecialty::Network),
            JobDiscipline::Programmer,
        ),
        (
            "Associate Software Engineer",
            JobLevel::Entry,
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Game Development Software Engineer",
            JobLevel::Mid,
            Some(JobSpecialty::Gameplay),
            JobDiscipline::Programmer,
        ),
        (
            "Software Engineering",
            JobLevel::Mid,
            None,
            JobDiscipline::Programmer,
        ),
        ("Modeler", JobLevel::Mid, None, JobDiscipline::Artist),
        (
            "FrontEnd Web Developer - EA Sports College Football (12 month temporary contract)",
            JobLevel::Mid,
            Some(JobSpecialty::Web),
            JobDiscipline::Programmer,
        ),
    ];
}
