module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  
  // Test file patterns
  roots: ['<rootDir>/src'],
  testMatch: [
    '**/__tests__/**/*.test.ts',
    '**/?(*.)+(spec|test).ts'
  ],
  
  // Coverage configuration
  collectCoverageFrom: [
    'src/**/*.ts',
    '!src/**/*.d.ts',
    '!src/setup/**',
    '!src/utils/test-helpers.ts'
  ],
  coverageDirectory: 'coverage',
  coverageReporters: ['text', 'lcov', 'html', 'json'],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  },
  
  // Reporters for different output formats
  reporters: [
    'default',
    [
      'jest-html-reporter',
      {
        pageTitle: 'RainbowBrowserAI Test Report',
        outputPath: './reports/test-report.html',
        includeFailureMsg: true,
        includeSuiteFailure: true
      }
    ],
    [
      'jest-junit',
      {
        outputDirectory: './reports',
        outputName: 'junit.xml',
        suiteName: 'RainbowBrowserAI Tests'
      }
    ]
  ],
  
  // Setup files
  setupFilesAfterEnv: ['<rootDir>/src/setup/simple-setup.ts'],
  
  // Timeouts
  testTimeout: 30000,
  
  // Module mapping for easier imports
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
    '^@test/(.*)$': '<rootDir>/src/utils/$1'
  },
  
  // Transform configuration
  transform: {
    '^.+\\.ts$': 'ts-jest'
  },
  
  // Verbose output
  verbose: true,
  
  // Clear mocks between tests
  clearMocks: true,
  
  // Collect coverage from all files
  collectCoverage: false, // Enable with --coverage flag
  
  // Global test variables
  globals: {
    'ts-jest': {
      tsconfig: 'tsconfig.json'
    }
  }
};