        assert_eq!(ErrorCode::E0001.severity(), Severity::Error);
        assert_eq!(ErrorCode::E1013.severity(), Severity::Warning);
    }

    #[test]
    fn test_display() {
        let code = ErrorCode::E2023;
        let display = format!("{code}");
        assert!(display.contains("E2023"));
        assert!(display.contains("undefined variable"));
    }

    #[test]
    fn test_suggestions_not_empty() {
        let suggestions = ErrorCode::E2023.suggestions();
        assert_ne!(suggestions, []);
    }

    #[test]
    fn test_explanation_not_empty() {
        let explanation = ErrorCode::E2023.explanation();
        assert_ne!(explanation, "");
        assert!(explanation.contains("declare"));
    }
}