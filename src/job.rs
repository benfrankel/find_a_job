use std::fmt::Display;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A job posting.
#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub source: String,
    pub title: String,
    pub level: JobLevel,
    pub specialty: Option<JobSpecialty>,
    pub discipline: JobDiscipline,
    pub is_general_application: bool,
    pub first_seen: DateTime<Utc>,
}

impl Display for Job {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.title)
    }
}

impl Job {
    pub fn new(title: impl Into<String>) -> Self {
        let title = title.into();
        let norm = normalized(&title);

        Self {
            source: String::new(),
            title,
            level: parse_level(&norm),
            specialty: parse_specialty(&norm),
            discipline: parse_discipline(&norm),
            is_general_application: parse_is_general_application(&norm),
            first_seen: Utc::now(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.title.len() <= 5
    }

    // TODO: Load preferences from a config file.
    pub fn is_good(&self) -> bool {
        !self.is_empty()
            && !self.is_general_application
            && [JobLevel::Entry, JobLevel::Mid].contains(&self.level)
            && self.discipline == JobDiscipline::Programmer
            && self.specialty.as_ref().is_none_or(|x| {
                [
                    JobSpecialty::Gameplay,
                    JobSpecialty::Graphics,
                    JobSpecialty::Engine,
                    JobSpecialty::Physics,
                ]
                .contains(&x)
            })
    }

    pub(crate) fn log_level(&self) -> log::Level {
        if self.is_good() {
            log::Level::Info
        } else {
            log::Level::Debug
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
    ($name:ident, $str:expr $(,)?) => {
        static $name: std::sync::LazyLock<regex::Regex> =
            std::sync::LazyLock::new(|| regex::Regex::new($str).unwrap());
    };
}

fn parse_level(norm: &str) -> JobLevel {
    re!(
        INTERN_RE,
        r"\b(intern(ship)?|co ?op|(under)?graduate|thesis)\b",
    );
    re!(ENTRY_RE, r"\b(entry|associate|junior|jr)\b");
    re!(MID_RE, r"\b(mid|executive assistant)\b");
    re!(
        SENIOR_RE,
        r"\b(senior|sr|expert|advanced?|principal|staff)\b",
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
        r"\b(automation|build|security|devops?|test(ing)?|site reliability|sre|platforms? engineer(ing)?)\b",
    );
    re!(WEB_RE, r"\b(web|front ?end)\b");
    re!(
        GRAPHICS_RE,
        r"\b(graphics|rendering|art|technical artist)\b",
    );
    re!(ANIMATION_RE, r"\b(animation)\b");
    re!(PHYSICS_RE, r"\b(physics)\b");
    re!(AUDIO_RE, r"\b(audio)\b");
    re!(AI_RE, r"\b(ai|computer vision|machine learning)\b");
    re!(UI_RE, r"\b(ui|ux|user interface|user experience)\b");
    re!(NETWORK_RE, r"\b(network|online|server|services?|backend)\b");
    re!(ENGINE_RE, r"\b(engine programmer|tools|technology)\b");
    re!(GAMEPLAY_RE, r"\b(gameplay)\b");

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
    re!(KNOWN_OTHER_RE, r"\b(bi engineer|support engineer)\b");
    re!(
        PROGRAMMER_RE,
        r"\b(programmer|coder|developer|engineer(ing)?|technical artist|swe|sre)\b",
    );
    re!(
        OTHER_RE,
        r"\b(specialist|researcher|scientist|analyst|assistant|responder|publishing|marketing|support)\b",
    );
    re!(ARTIST_RE, r"\b(artist|animator|modeler)\b");
    re!(WRITER_RE, r"\b(writer)\b");
    re!(COMPOSER_RE, r"\b(composer)\b");
    re!(DESIGNER_RE, r"\b(designer|architect)\b");
    re!(SNEAKY_MANAGER_RE, r"\b(lead|head)\b");

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
    } else if SNEAKY_MANAGER_RE.is_match(norm) {
        JobDiscipline::Manager
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
    fn is_empty() {
        for s in ["", " ", "-", "N/a", "None"] {
            assert!(Job::new(s).is_empty(), "{}", s);
        }
    }

    #[test]
    fn level() {
        for (title, level, _, _) in TEST_CASES {
            assert_eq!(Job::new(title).level, level, "{}", title);
        }
    }

    #[test]
    fn specialty() {
        for (title, _, specialty, _) in TEST_CASES {
            assert_eq!(Job::new(title).specialty, specialty, "{}", title);
        }
    }

    #[test]
    fn discipline() {
        for (title, _, _, discipline) in TEST_CASES {
            assert_eq!(Job::new(title).discipline, discipline, "{}", title);
        }
    }

    #[test]
    fn is_general_application() {
        for s in ["Engineering Application Drop Box", "General Application"] {
            assert!(Job::new(s).is_general_application, "{}", s);
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
            None,
            JobDiscipline::Programmer,
        ),
        (
            "Senior/Lead C++ Software Engineer (Generalist - Game Modes) - American Football",
            JobLevel::Senior,
            None,
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
            Some(JobSpecialty::Ai),
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
            None,
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
            None,
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
            None,
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
